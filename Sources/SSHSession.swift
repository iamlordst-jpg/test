import Foundation
import UIKit
import SwiftTerm
import Citadel
import NIOCore
import NIOSSH

final class SSHSession: NSObject, ObservableObject, TerminalViewDelegate {
    weak var term: TerminalView?
    private var client: SSHClient?
    private var writer: TTYStdinWriter?
    private var task: Task<Void, Never>?

    func connect(host: String, port: Int, user: String, password: String, tmux: Bool) {
        task = Task {
            do {
                let c = try await SSHClient.connect(
                    host: host, port: port,
                    authenticationMethod: .passwordBased(username: user, password: password),
                    hostKeyValidator: .acceptAnything(),
                    reconnect: .never)
                self.client = c
                let req = SSHChannelRequestEvent.PseudoTerminalRequest(
                    wantReply: true, term: "xterm-256color",
                    terminalCharacterWidth: 80, terminalRowHeight: 24,
                    terminalPixelWidth: 0, terminalPixelHeight: 0,
                    terminalModes: .init([.ECHO: 1]))
                try await c.withPTY(req) { inbound, outbound in
                    self.writer = outbound
                    if tmux {
                        try await outbound.write(ByteBuffer(string: "tmux new -A -s main\n"))
                    }
                    for try await chunk in inbound {
                        switch chunk {
                        case .stdout(let b), .stderr(let b):
                            let bytes = Array(b.readableBytesView)
                            await MainActor.run { self.term?.feed(byteArray: bytes[...]) }
                        }
                    }
                }
            } catch {
                await MainActor.run {
                    self.term?.feed(text: "\r\n[connection error] \(error)\r\n")
                }
            }
        }
    }

    func disconnect() {
        task?.cancel()
        Task { try? await client?.close() }
    }

    func send(_ bytes: [UInt8]) {
        guard let w = writer else { return }
        Task { try? await w.write(ByteBuffer(bytes: bytes)) }
    }

    // MARK: TerminalViewDelegate
    func send(source: TerminalView, data: ArraySlice<UInt8>) { send(Array(data)) }
    func sizeChanged(source: TerminalView, newCols: Int, newRows: Int) {
        guard let w = writer else { return }
        Task { try? await w.changeSize(cols: newCols, rows: newRows, pixelWidth: 0, pixelHeight: 0) }
    }
    func setTerminalTitle(source: TerminalView, title: String) {}
    func hostCurrentDirectoryUpdate(source: TerminalView, directory: String?) {}
    func scrolled(source: TerminalView, position: Double) {}
    func requestOpenLink(source: TerminalView, link: String, params: [String: String]) {}
    func clipboardCopy(source: TerminalView, content: Data) {
        UIPasteboard.general.string = String(data: content, encoding: .utf8)
    }
    func rangeChanged(source: TerminalView, startY: Int, endY: Int) {}
    func iTermContent(source: TerminalView, content: ArraySlice<UInt8>) {}
}
