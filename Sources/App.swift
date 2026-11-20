import SwiftUI

@main
struct ClaudePocketApp: App {
    var body: some Scene {
        WindowGroup { RootView() }
    }
}

struct RootView: View {
    @AppStorage("host") var host = ""
    @AppStorage("port") var port = "22"
    @AppStorage("user") var user = ""
    @State private var password = ""
    @State private var connected = false
    @State private var useTmux = true

    var body: some View {
        if connected {
            TerminalScreen(host: host, port: Int(port) ?? 22, user: user,
                           password: password, useTmux: useTmux) { connected = false }
        } else {
            NavigationStack {
                Form {
                    Section("Server") {
                        TextField("Host / IP", text: $host)
                            .textInputAutocapitalization(.never).autocorrectionDisabled()
                        TextField("Port", text: $port).keyboardType(.numberPad)
                        TextField("User", text: $user)
                            .textInputAutocapitalization(.never).autocorrectionDisabled()
                        SecureField("Password", text: $password)
                        Toggle("Auto-attach tmux", isOn: $useTmux)
                    }
                    Button("Connect") { connected = true }
                        .disabled(host.isEmpty || user.isEmpty)
                }
                .navigationTitle("ClaudePocket")
            }
        }
    }
}
