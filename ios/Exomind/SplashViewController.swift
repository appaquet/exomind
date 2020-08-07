
import UIKit

class SplashViewController: UIViewController {
    @IBAction func skipLogin(_ sender: AnyObject) {
        (self.navigationController as? RootNavigationController)?.showLogin()
    }
}
