import UIKit
import FontAwesome_swift

class TabBarController: UITabBarController {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    fileprivate let homeIndex = 0
    fileprivate var homeItem: UITabBarItem!
    fileprivate var homeVC: UIViewController!
    fileprivate let settingsIndex = 1
    fileprivate var settingsItem: UITabBarItem!
    fileprivate var settingsVC: UIViewController!

    fileprivate let imageHome = UIImage.fontAwesomeIcon(name: .home, style: .solid, textColor: UIColor.black, size: CGSize(width: 28, height: 28))
    fileprivate let imageSettings = UIImage.fontAwesomeIcon(name: .cog, style: .solid, textColor: UIColor.black, size: CGSize(width: 28, height: 28))

    fileprivate var objectsController = [String: NavigationController]()

    override func viewDidLoad() {
        super.viewDidLoad()

        self.homeItem = self.tabBar.items![self.homeIndex]
        self.homeItem.image = imageHome
        self.homeVC = self.viewControllers![self.homeIndex]

        self.settingsItem = self.tabBar.items![self.settingsIndex]
        self.settingsItem.image = imageSettings
        self.settingsVC = self.viewControllers![self.settingsIndex]

        self.tabBar.tintColor = Stylesheet.tabBarSelectedFg
        
        // make sure that appearance with or without data under tabbar is the same since,
        // from iOS 15, tab bar may be transparent when no data is under
        // https://nemecek.be/blog/127/how-to-disable-automatic-transparent-tabbar-in-ios-15
        self.tabBar.scrollEdgeAppearance = self.tabBar.standardAppearance

        // remove title and offset items because of created extra space
        for item in self.tabBar.items! {
            item.imageInsets = UIEdgeInsets(top: 6, left: 0, bottom: -6, right: 0)
            item.title = ""
        }
    }

    // TODO: Middle item
    // https://medium.com/better-programming/how-to-create-a-custom-action-for-center-tab-bar-item-65e3e5cb0519

    func show(navigationObject: NavigationObject) {
        self.selectedIndex = 0
        (self.selectedViewController as? NavigationController)?.pushObject(navigationObject, animated: true)
    }

    deinit {
        print("NavigationController > Deinit")
    }
}
