
import UIKit
import FontAwesome_swift

class TabBarController: UITabBarController {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    fileprivate var querySet: QuerySet!

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

        self.loadData()
    }

    func loadData() {
        if (self.querySet == nil) {
            self.querySet = DomainStore.instance.getQuerySet()
            self.querySet.onChange({
                [weak self] () -> Void in
                self?.loadData()
            })
        }
        
        let mindChildrenQuery = self.querySet.executeQuery(HCQueries.Entities().withParent(entityId: "mind"))
        if (mindChildrenQuery.isLoaded()) {
            var newMap = [String: NavigationController]();
            var barViews: [UIViewController] = [self.inboxVC]
            for entity in mindChildrenQuery.resultsAsEntities() {
                if let entityTrait = EntityTraitOld(entity: entity) {
                    switch (entityTrait.traitType) {
                    case .inbox(inbox: _), .mind(mind: _):
                        continue
                    default:
                        if let vc = objectsController[entity.id] {
                            barViews.append(vc)
                            newMap[entity.id] = vc
                        } else {
//                            let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "NavigationController") as! NavigationController
//                            vc.replaceTopObject(.entityOld(entity: entity))
//                            let image = ObjectsIcon.icon(forEntity: entity, color: UIColor.black, dimension: 28)
//                            vc.tabBarItem = UITabBarItem(title: entityTrait.displayName, image: image, tag: 0)
//                            barViews.append(vc)
//                            newMap[entity.id] = vc
                        }
                    }
                }
            }
            objectsController = newMap
            barViews.append(self.settingsVC)
            self.setViewControllers(barViews, animated: true)
        }
    }
    
    func show(navigationObject: NavigationObject) {
        self.selectedIndex = 0
        (self.selectedViewController as? NavigationController)?.pushObject(navigationObject, animated: true)
    }

    deinit {
        print("NavigationController > Deinit")
    }
}
