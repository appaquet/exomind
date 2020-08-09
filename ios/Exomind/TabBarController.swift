import UIKit
import FontAwesome_swift

class TabBarController: UITabBarController {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    fileprivate let inboxIndex = 0
    fileprivate var inboxItem: UITabBarItem!
    fileprivate var inboxVC: UIViewController!
    fileprivate let settingsIndex = 1
    fileprivate var settingsItem: UITabBarItem!
    fileprivate var settingsVC: UIViewController!

    fileprivate let imageInbox = UIImage.fontAwesomeIcon(name: .inbox, style: .solid, textColor: UIColor.black, size: CGSize(width: 28, height: 28))
    fileprivate let imageSettings = UIImage.fontAwesomeIcon(name: .cog, style: .solid, textColor: UIColor.black, size: CGSize(width: 28, height: 28))

    fileprivate var objectsController = [String: NavigationController]()

    override func viewDidLoad() {
        super.viewDidLoad()

        self.inboxItem = self.tabBar.items![self.inboxIndex]
        self.inboxItem.image = imageInbox
        self.inboxVC = self.viewControllers![self.inboxIndex]
        self.settingsItem = self.tabBar.items![self.settingsIndex]
        self.settingsItem.image = imageSettings
        self.settingsVC = self.viewControllers![self.settingsIndex]

        self.tabBar.tintColor = Stylesheet.tabBarSelectedFg
    }

    func show(navigationObject: NavigationObject) {
        self.selectedIndex = 0
        (self.selectedViewController as? NavigationController)?.pushObject(navigationObject, animated: true)
    }

    deinit {
        print("NavigationController > Deinit")
    }
}
