import UIKit

class RootViewController: UIViewController {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    private var currentView: UIViewController?

    static func mainInstance() -> RootViewController? {
        let app = UIApplication.shared
        let vc = app.windows[0].rootViewController as? RootViewController
        return vc
    }

    override func viewDidLoad() {
        self.showStateView()
    }

    func show(navigationObject: NavigationObject) {
        if let tabBar = self.currentView as? TabBarController {
            tabBar.show(navigationObject: navigationObject)
        }
    }

    func showBootstrap() {
        let onBootstrap = self.currentView as? BootstrapViewController != nil
        if (!onBootstrap) {
            let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "bootstrapViewController") as! BootstrapViewController
            vc.onDone = { [weak self] in
                self?.showStateView()
            }
            self.changeVC(vc)
        }
    }

    private func showStateView() {
        if !ExocoreUtils.nodeHasCell {
            self.showBootstrap()
        } else {
            self.showTabBar()
        }
    }

    private func showTabBar() {
        let onTabBar = self.currentView as? TabBarController != nil
        if (!onTabBar) {
            let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "tabBarViewController")
            self.changeVC(vc)
            Notifications.maybeRegister()
        }
    }

    private func changeVC(_ vc: UIViewController) {
        self.currentView?.removeFromParent()
        self.currentView = nil

        self.addChild(vc)
        vc.view.frame = CGRect(x: 0, y: 0, width: self.view.frame.size.width, height: self.view.frame.size.height);
        self.view.addSubview(vc.view)
        vc.didMove(toParent: self)
        vc.viewWillAppear(false)
        vc.viewDidAppear(false)
    }
}
