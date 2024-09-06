
import SwiftUI

struct MainView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        VStack {
            if appState.currentView == .discovery {
                DiscoveryView()
            } else if appState.currentView == .list {
                ListView()
            }
        }
    }
}

struct MainView_Previews: PreviewProvider {
    static var previews: some View {
        MainView().environmentObject(AppState())
    }
}
