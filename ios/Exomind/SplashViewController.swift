import UIKit

class SplashViewController: UIViewController {
    @IBAction func showBootstrap(_ sender: AnyObject) {
        (self.navigationController as? RootNavigationController)?.showBootstrap(fromRoot: true)
    }
}
