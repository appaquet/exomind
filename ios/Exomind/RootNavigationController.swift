import UIKit

class RootNavigationController: UINavigationController {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    static func mainInstance() -> RootNavigationController? {
        let app = UIApplication.shared
        let vc = app.windows[0].rootViewController as? RootNavigationController
        return vc
    }

    override func viewDidLoad() {
        self.showStateView()
    }

    @objc func showStateView() {
        if !ExocoreUtils.nodeHasCell {
            self.showBootstrap(fromRoot: true)
        } else {
            self.showTabBar()
        }
    }

    func showTabBar() {
        let onTabBar = self.topViewController as? TabBarController != nil
        if (!onTabBar) {
            self.popToRootViewController(animated: false)
            let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "tabBarViewController")
            self.show(vc, sender: self)
            NotificationsController.maybeRegister()
        }
    }

    func showBootstrap(fromRoot: Bool) {
        let onBootstrap = self.topViewController as? BootstrapViewController != nil
        if (!onBootstrap) {
            if fromRoot {
                self.popToRootViewController(animated: false)
            }

            let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "bootstrapViewController") as! BootstrapViewController
            vc.onDone = { [weak self] in
                self?.popToRootViewController(animated: false)
                self?.showStateView()
            }
            self.show(vc, sender: self)
        }
    }

    func showSplash() {
        let onSplash = self.topViewController as? SplashViewController != nil
        if (!onSplash) {
            self.popToRootViewController(animated: false)
        }
    }

    func show(navigationObject: NavigationObject) {
        if let tabBar = self.topViewController as? TabBarController {
            tabBar.show(navigationObject: navigationObject)
        }
    }
}
