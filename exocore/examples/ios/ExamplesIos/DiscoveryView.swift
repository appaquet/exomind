import SwiftUI
import Exocore

struct DiscoveryView: View {
    @EnvironmentObject var appState: AppState
    @StateObject var state: DiscoveryState = DiscoveryState()

    var body: some View {
        NavigationView {
            VStack {
                discoView()
                Spacer()
                errorView()
                Spacer()
                Button("Generate new node") {
                    appState.node = try? LocalNode.generate()
                    appState.refreshNodeConfig()
                    state.performJoinCell(appState)
                }
            }
            .navigationBarTitle("Cell discovery")
            .onAppear {
                state.performJoinCell(appState)
            }
        }
    }

    @ViewBuilder
    func discoView() -> some View {
        if let pin = state.pin {
            VStack {
                Text("Discovery PIN:")
                    .padding()
                Text(formatPin(pin))
                    .bold()
                    .font(.system(size: 30))
            }
        } else {
            Text("Joining...")
        }
    }

    func errorView() -> Text? {
        if let err = self.state.error {
            return Text("Error: \(err)").foregroundColor(.red)
        }

        return appState.currentError.map {
            Text("Error: \($0)").foregroundColor(.red)
        }
    }
}

class DiscoveryState: ObservableObject {
    private var discovery: Discovery?
    @Published var pin: UInt32?
    @Published var error: String?

    func performJoinCell(_ appState: AppState) {
        guard let node = appState.node else {
            self.error = "No node configured"
            return
        }

        do {
            let discovery = try Discovery()
            self.discovery = discovery
            self.pin = nil
            self.error = nil

            discovery.joinCell(localNode: node) { [weak self] (stage) in
                DispatchQueue.main.async {
                    switch stage {
                    case .pin(let pin):
                        self?.pin = pin
                    case .success(let newNode):
                        appState.node = newNode
                        appState.forceDiscovery = false
                        appState.refreshNodeConfig()
                    case .error(let err):
                        self?.error = err.localizedDescription
                    }
                }
            }
        } catch {
            self.error = error.localizedDescription
        }
    }
}

private func formatPin(_ pin: UInt32) -> String {
    let strPin = pin.description

    var ret = ""
    for (i, char) in strPin.enumerated() {
        if i == 3 || i == 6 {
            ret += " "
        }
        ret += String(char)
    }
    return ret
}

#if DEBUG
struct BootstrapView_Previews: PreviewProvider {
    static var previews: some View {
        DiscoveryView()
    }
}
#endif
