import SwiftUI
import SwiftTerm

struct TerminalScreen: View {
    let host: String; let port: Int; let user: String
    let password: String; let useTmux: Bool
    let onClose: () -> Void
    @StateObject private var session = SSHSession()

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("\(user)@\(host)").font(.caption).lineLimit(1)
                Spacer()
                Button("Close", action: onClose).font(.caption)
            }.padding(8)
            TerminalRepresentable(session: session)
            ScrollView(.horizontal, showsIndicators: false) {
                HStack {
                    key("Esc", [0x1b]); key("Tab", [0x09])
                    key("Ctrl-C", [0x03]); key("Ctrl-D", [0x04])
                    key("↑", [0x1b, 0x5b, 0x41]); key("↓", [0x1b, 0x5b, 0x42])
                    key("←", [0x1b, 0x5b, 0x44]); key("→", [0x1b, 0x5b, 0x43])
                    key("Yes", Array("y\n".utf8)); key("No", Array("n\n".utf8))
                }.padding(6)
            }
        }
        .background(Color.black.ignoresSafeArea(edges: .bottom))
        .onAppear { session.connect(host: host, port: port, user: user, password: password, tmux: useTmux) }
        .onDisappear { session.disconnect() }
    }

    private func key(_ label: String, _ bytes: [UInt8]) -> some View {
        Button(label) { session.send(bytes) }
            .buttonStyle(.bordered).font(.caption)
    }
}

struct TerminalRepresentable: UIViewRepresentable {
    let session: SSHSession
    func makeUIView(context: Context) -> TerminalView {
        let tv = TerminalView(frame: .zero)
        tv.terminalDelegate = session
        session.term = tv
        tv.becomeFirstResponder()
        return tv
    }
    func updateUIView(_ uiView: TerminalView, context: Context) {}
}
