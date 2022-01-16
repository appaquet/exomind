import UIKit
import FontAwesome_swift

class NavigationController: UINavigationController, UINavigationControllerDelegate {
    fileprivate let objectsStoryboard: UIStoryboard = UIStoryboard(name: "Objects", bundle: nil)
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    fileprivate var quickButton: QuickButtonView!
    fileprivate static let quickButtonExtraMargin = CGFloat(10)
    fileprivate var barActions: [NavigationControllerBarAction] = []

    override func viewDidLoad() {
        super.viewDidLoad()
        self.delegate = self

        // set colors of navigation bar
        Stylesheet.styleNavigationBar(self.navigationBar, bgColor: Stylesheet.navigationBarBg, fgColor: Stylesheet.navigationBarFg)

        // quick button at bottom
        self.quickButton = QuickButtonView()
        var bottomMargin = NavigationController.quickButtonExtraMargin
        if let tabBar = self.tabBarController {
            bottomMargin = bottomMargin + tabBar.tabBar.frame.height
        }
        self.quickButton.addToView(self.view, bottomMargin: bottomMargin)
    }

    override func viewLayoutMarginsDidChange() {
        if let tabBar = self.tabBarController {
            self.quickButton.setBottomMargin(NavigationController.quickButtonExtraMargin + tabBar.tabBar.frame.height)
        }
    }

    func replaceTopObject(_ object: NavigationObject) {
        let vc = self.createViewController(forObject: object)
        self.setViewControllers([vc], animated: true)
    }

    func pushObject(_ object: NavigationObject, animated: Bool = true) {
        var entityTrait: AnyTraitInstance?
        switch (object) {
        case let .entity(entity: entity):
            entityTrait = entity.priorityTrait
        case let .entityTrait(entityTrait: et):
            entityTrait = et
        default:
            entityTrait = nil
        }

        if let et = entityTrait,
           case let .link(trait: link) = et.typeInstance(),
           let url = URL(string: link.message.url) {

            let sfVc = SFSafariHelper.getViewControllerForURL(url)
            self.present(sfVc, animated: true, completion: nil)
        } else {
            let vc = createViewController(forObject: object)
            self.pushViewController(vc, animated: animated)
        }
    }

    func pushInbox(_ animated: Bool = true) {
        let vc = InboxViewController()
        self.pushViewController(vc, animated: animated)
    }

    func pushSnoozed(_ animated: Bool = true) {
        let vc = SnoozedViewController()
        self.pushViewController(vc, animated: animated)
    }

    func pushRecent(_ animated: Bool = true) {
        let vc = RecentViewController()
        self.pushViewController(vc, animated: animated)
    }

    private func createViewController(forObject: NavigationObject) -> UIViewController {
        switch (forObject) {
        case let .entityId(id: entityId) where entityId == "inbox":
            let vc = InboxViewController()
            return vc

        case let .entity(entity: entity) where entity.id == "inbox":
            let vc = InboxViewController()
            return vc

        case let .entityId(id: entityId):
            let vc = objectsStoryboard.instantiateViewController(withIdentifier: "EntityViewController") as! EntityViewController
            vc.populate(entityId: entityId)
            return vc

        case let .entity(entity: entity):
            let vc = objectsStoryboard.instantiateViewController(withIdentifier: "EntityViewController") as! EntityViewController
            vc.populate(entity: entity)
            return vc

        case let .entityTrait(entityTrait: et):
            let vc = objectsStoryboard.instantiateViewController(withIdentifier: "EntityViewController") as! EntityViewController
            vc.populate(entityTrait: et)
            return vc
        }
    }

    func resetState() {
        self.topViewController?.navigationItem.rightBarButtonItems = []
        self.clearBarActions()
        self.setQuickButtonVisibility(false)
    }

    func setQuickButtonActions(_ actions: [QuickButtonAction]) {
        self.quickButton.setActions(actions)
        self.quickButton.isHidden = false
    }

    func setQuickButtonVisibility(_ shown: Bool) {
        self.quickButton.close()
        self.quickButton.isHidden = !shown
    }

    func setBarActions(_ actions: [NavigationControllerBarAction]) {
        self.barActions = actions.reversed()
        self.topViewController?.navigationItem.rightBarButtonItems? = self.barActions.enumerated().map {
            (i, action) in
            let color = action.active ? Stylesheet.navigationBarActiveFg : Stylesheet.navigationBarFg
            let img = UIImage.fontAwesomeIcon(name: action.icon, style: .solid, textColor: color, size: CGSize(width: 25, height: 25))
            let button = UIButton()
            button.setImage(img, for: UIControl.State())
            button.frame = CGRect(x: 0, y: 0, width: 25, height: 25)
            button.tag = i
            button.addTarget(self, action: #selector(handleBarActionClick), for: .touchUpInside)
            let barButton = UIBarButtonItem()
            barButton.customView = button
            return barButton
        }
    }

    func clearBarActions() {
        self.topViewController?.navigationItem.rightBarButtonItems? = []
    }

    @objc func handleBarActionClick(_ sender: UIButton) {
        self.barActions[sender.tag].handler?()
    }

    func showCollectionSelector(forEntity: EntityExt) {
        let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "CollectionSelectorViewController") as! CollectionSelectorViewController
        vc.forEntity = forEntity
        self.present(vc, animated: true, completion: nil)
    }

    func showSearch(_ fromEntityId: EntityId?, selectionHandler: ((EntityExt) -> Void)? = nil) {
        let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "SearchViewController") as! SearchViewController
        vc.fromEntityId = fromEntityId
        if let handler = selectionHandler {
            vc.selectionHandler = handler
        }
        self.present(vc, animated: true, completion: nil)
    }

    func showCreateObject(_ fromEntityId: EntityId, callback: ((EntityCreateResult) -> Void)? = nil) {
        let showIn = self.parent ?? self
        let vc = EntityCreationViewController(parentId: fromEntityId, callback: callback)
        vc.showInsideViewController(showIn)
    }

    func showTimeSelector(forEntity: EntityExt, callback: ((Bool) -> Void)? = nil) {
        let showIn = self.parent ?? self
        let timeSelector = TimeSelectionViewController { (date) in
            if let realDate = date {
                Commands.snooze(entity: forEntity, date: realDate)
                callback?(true)
            } else {
                callback?(false)
            }
        }
        timeSelector.showInsideViewController(showIn)
    }
}

class NavigationControllerBarAction {
    let icon: FontAwesome
    let handler: (() -> Void)?
    let active: Bool

    init(icon: FontAwesome, active: Bool = false, handler: (() -> Void)?) {
        self.icon = icon
        self.handler = handler
        self.active = active
    }
}

enum NavigationObject {
    case entity(entity: EntityExt)
    case entityTrait(entityTrait: AnyTraitInstance)
    case entityId(id: EntityId)
}
