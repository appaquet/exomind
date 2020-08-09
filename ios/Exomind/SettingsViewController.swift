import UIKit
import Alamofire
import KeychainSwift

class SettingsViewController: UITableViewController {
    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        switch ((indexPath as NSIndexPath).section, (indexPath as NSIndexPath).item) {
        case (0, 0): // edit favorites
            self.exomindBar()

        case (1, 0): // cell config
            self.cellConfig()

        default:
            print("SettingsViewController > Unhandled setting \(indexPath)")
        }
    }

    func exomindBar() {
//        if let mind = SessionStore.mindEntity() {
//            (self.tabBarController as? TabBarController)?.show(navigationObject: .entityOld(entity: mind))
//        }
    }

    func cellConfig() {
        RootNavigationController.mainInstance()?.showBootstrap(fromRoot: false)
    }

    func logout() {
        Alamofire
                .request("https://exomind.io/logout")
                .response { (resp) in
                    print("SettingsViewController > Logged out")
                }

        HttpUtils.wipeCookies()
        JSBridge.instance.resetConnections()
    }
}
