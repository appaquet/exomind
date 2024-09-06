import UIKit

class RootViewController: UIViewController {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    private var currentView: UIViewController?

    static func mainInstance() -> RootViewController? {
        guard let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene else {
            return nil
        }
        
        let vc = windowScene.windows.first?.rootViewController as? RootViewController
        return vc
    }

    override func viewDidLoad() {
        self.showStateView()
        NotificationCenter.default.addObserver(self, selector: #selector(onCollectionsChanged), name: .exomindCollectionsChanged, object: nil)
    }

    func show(navigationObject: NavigationObject) {
        if let tabBar = self.currentView as? TabBarController {
            tabBar.show(navigationObject: navigationObject)
        }
    }

    func showBootstrap() {
        let onBootstrap = self.currentView as? BootstrapViewController != nil
        if (!onBootstrap) {
            print("RootViewController > Changing to bootstrap view")
            let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "bootstrapViewController") as! BootstrapViewController
            vc.onDone = { [weak self] in
                self?.showStateView()
            }
            self.changeVC(vc)
        }
    }

    @objc private func onCollectionsChanged() {
        DispatchQueue.main.async {
            self.showStateView()
        }
    }

    private func showStateView() {
        if !ExocoreUtils.nodeHasCell {
            self.showBootstrap()
        } else {
            self.showApplication()
        }
    }

    private func showApplication() {
        let onApplication = self.currentView as? TabBarController != nil
        if (!onApplication) {
            print("RootViewController > Changing to main application")
            let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "tabBarViewController")
            self.changeVC(vc)
            Notifications.maybeRegister()
        }
    }

    private func changeVC(_ vc: UIViewController) {
        self.currentView?.removeFromParent()
        self.currentView = vc

        self.addChild(vc)
        vc.view.frame = CGRect(x: 0, y: 0, width: self.view.frame.size.width, height: self.view.frame.size.height);
        self.view.addSubview(vc.view)
        vc.didMove(toParent: self)
        vc.viewWillAppear(false)
        vc.viewDidAppear(false)
    }
}
