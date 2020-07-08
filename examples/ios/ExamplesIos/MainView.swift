
import SwiftUI

struct MainView: View {
    @EnvironmentObject var viewRouter: AppState

    var body: some View {
        VStack {
            if viewRouter.currentView == .bootstrap {
                BootstrapView()
            } else if viewRouter.currentView == .list {
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
