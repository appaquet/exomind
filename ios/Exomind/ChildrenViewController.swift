
import UIKit
import FontAwesome_swift
import MCSwipeTableViewCell
import Dwifft
import Exocore

class ChildrenViewController: UITableViewController {
    private var query: ExpandableQuery?
    private var parentId: EntityId?

    private var childrenType: String = "current"

    private var collectionData: [Exocore_Index_EntityResult] = []
    private var itemClickHandler: ((EntityExt) -> Void)?
    private var swipeActions: [ChildrenViewSwipeAction] = []

    private var switcherButton: SwitcherButton?
    private var switcherButtonActions: [SwitcherButtonAction] = []

    private var scrollEverDragged = false
    private var scrollDragging = false
    private var headerShown: Bool = false
    private var headerWasShownBeforeDrag: Bool = false
    private var currentlyExpandingQuery = false

    var diffCalculator: SingleSectionTableViewDiffCalculator<Exocore_Index_EntityResult>?

    override func viewDidLoad() {
        super.viewDidLoad()
        self.tableView.delegate = self
        self.diffCalculator = SingleSectionTableViewDiffCalculator<Exocore_Index_EntityResult>(tableView: self.tableView, initialRows: [])
        self.diffCalculator?.insertionAnimation = .fade
        self.diffCalculator?.deletionAnimation = .fade
    }

    func setSwipeActions(_ actions: [ChildrenViewSwipeAction]) {
        self.swipeActions = actions
    }

    func setItemClickHandler(_ handler: @escaping (EntityExt) -> Void) {
        self.itemClickHandler = handler
    }

    func setSwitcherActions(_ actions: [SwitcherButtonAction]) {
        if let headerView = self.tableView.tableHeaderView {
            if (self.switcherButton == nil) {
                let switcherButton = SwitcherButton(frame: CGRect())
                self.switcherButton = switcherButton
                headerView.addSubview(switcherButton)
                headerView.addConstraints([
                    NSLayoutConstraint(item: switcherButton, attribute: .centerX, relatedBy: .equal, toItem: headerView, attribute: .centerX, multiplier: 1, constant: 0),
                    NSLayoutConstraint(item: switcherButton, attribute: .centerY, relatedBy: .equal, toItem: headerView, attribute: .centerY, multiplier: 1, constant: 0)
                ])
            }

            self.switcherButtonActions = actions
            self.switcherButton?.setActions(actions)
        }
    }

    func loadData(withResults results: [Exocore_Index_EntityResult]) {
        self.collectionData = results
    }

    func loadData(fromChildrenOf entityId: EntityId) {
        let traitQuery = TraitQueryBuilder.refersTo(field: "collection", entityId: entityId).build()
        let query = QueryBuilder
                .withTrait(Exomind_Base_CollectionChild.self, query: traitQuery)
                .orderByField("weight", ascending: false)
                .count(30)
                .build()

        self.parentId = entityId
        self.loadData(fromQuery: query)
    }

    func loadData(fromQuery query: Exocore_Index_EntityQuery) {
        self.query = ExpandableQuery(query: query) { [weak self] in
            guard let this = self else {
                return
            }

            DispatchQueue.main.async {
                this.collectionData = this.query?.results ?? []
                this.diffCalculator?.rows = this.collectionData
            }
        }
    }

    func hasHeader() -> Bool {
        self.switcherButtonActions.count > 0
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        var top = CGFloat(0.0)
        let bottom = Stylesheet.quickButtonSize + 20
        let showHeader = self.hasHeader() && (scrollDragging && headerWasShownBeforeDrag) || (!scrollDragging && headerShown)
        if (!showHeader) {
            top = top - (self.tableView.tableHeaderView?.frame.height ?? 0)
        }
        let newInsets = UIEdgeInsets(top: top, left: 0, bottom: bottom, right: 0)
        self.tableView.contentInset = newInsets
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        self.collectionData.count
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = self.tableView.dequeueReusableCell(withIdentifier: "cell", for: indexPath) as! ChildrenViewCell

        // fixes borders between each cell that otherwise aren't 100% width
        cell.layoutMargins = UIEdgeInsets.zero
        cell.preservesSuperviewLayoutMargins = false

        self.configureCellSwipe(indexPath, cell: cell)
        cell.populate(self.collectionData[(indexPath as NSIndexPath).item])
        return cell
    }

    override func scrollViewDidScroll(_ scrollView: UIScrollView) {
        let headerHeight = (self.tableView.tableHeaderView?.frame.height ?? CGFloat(44))

        let totalHeight = (scrollView.contentSize.height - tableView.frame.size.height)
        let currentPosition = scrollView.contentOffset.y
        let averageHeight = CGFloat(74)
        let itemsComingUp = (totalHeight - currentPosition) / averageHeight

        // if we suddenly have more items, that means we are at beginning or expansion worked
        if (itemsComingUp > 10) {
            self.currentlyExpandingQuery = false
        }

        // if only 5 items or less are coming up, we load new
        if (itemsComingUp < 10 && self.collectionData.count > 0 && !self.currentlyExpandingQuery) {
            self.currentlyExpandingQuery = true
            self.query?.expand()
        }

        if (!headerShown && scrollEverDragged && scrollView.contentOffset.y < -headerHeight * 1.25) {
            self.headerShown = true
        } else if (headerShown && scrollView.contentOffset.y > 0 && scrollDragging) {
            self.headerShown = false
        }
    }

    override func scrollViewDidEndDragging(_ scrollView: UIScrollView, willDecelerate decelerate: Bool) {
        self.scrollDragging = false
        self.scrollEverDragged = true
    }

    override func scrollViewWillBeginDragging(_ scrollView: UIScrollView) {
        self.scrollDragging = true
        self.scrollEverDragged = true
        self.headerWasShownBeforeDrag = self.headerShown
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let res = self.collectionData[(indexPath as NSIndexPath).item]
        if let handler = self.itemClickHandler {
            handler(res.entity.toExtension())
        }
    }

    func configureCellSwipe(_ indexPath: IndexPath, cell: ChildrenViewCell) -> Void {
        // background of the swipe cell
        cell.defaultColor = UIColor.systemBackground

        for action in self.swipeActions {
            let swipeIconView = UIView()
            let iconImgView = UIImageView(image: UIImage.fontAwesomeIcon(name: action.icon, style: .solid, textColor: UIColor.white, size: CGSize(width: 30, height: 30)))
            swipeIconView.addSubview(iconImgView)
            iconImgView.center = swipeIconView.center

            cell.setSwipeGestureWith(swipeIconView, color: action.color, mode: action.mode, state: action.state) { (scell, state, mode) -> Void in
                action.handler(cell.entity)
            }
        }
    }

    deinit {
        print("ChildrenViewController > Deinit")
    }
}

class ChildrenViewCell: MCSwipeTableViewCell {
    @IBOutlet weak var title1: UILabel!
    @IBOutlet weak var title2: UILabel!
    @IBOutlet weak var title3: UILabel!
    @IBOutlet weak var date: UILabel!
    @IBOutlet weak var icon: UIImageView!

    var entity: EntityExt!

    func populate(_ result: Exocore_Index_EntityResult) {
        self.backgroundColor = UIColor.clear

        self.entity = result.entity.toExtension()

        guard let priorityTrait = self.entity.priorityTrait else {
            self.title1.text = "UNKNOWN ENTITY TRAIT"
            return
        }

        let displayName = priorityTrait.displayName
        self.date.text = priorityTrait.modificationDate?.toShort() ?? priorityTrait.creationDate.toShort()

        self.title1.font = UIFont.systemFont(ofSize: 14)
        self.title2.font = UIFont.systemFont(ofSize: 14)
        self.title3.font = UIFont.systemFont(ofSize: 14)

        switch priorityTrait.typeInstance() {
        case let .email(email):
            self.title1.text = EmailsLogic.formatContact(email.message.from)
            self.title2.text = displayName
            self.title3.text = email.message.snippet

        case let .emailThread(emailThread):
            let emails = entity.traitsOfType(Exomind_Base_Email.self)

            self.title1.text = EmailsLogic.formatContact(emailThread.message.from)
            if emails.count > 1 {
                self.title1.text = "\(self.title1.text!) (\(emails.count))"
            }

            if !emailThread.message.read {
                self.title1.font = UIFont.boldSystemFont(ofSize: 14)
                self.title2.font = UIFont.boldSystemFont(ofSize: 14)
                self.title3.font = UIFont.boldSystemFont(ofSize: 14)
            }

            let lastEmail = emails.max(by: { (a, b) -> Bool in
                let aDate = a.modificationDate ?? a.creationDate
                let bDate = b.modificationDate ?? b.creationDate
                return aDate < bDate
            })

            if let lastEmail = lastEmail {
                self.date.text = lastEmail.modificationDate?.toShort() ?? lastEmail.creationDate.toShort()
            }

            self.title2.text = displayName
            self.title3.text = emailThread.message.snippet

        case let .draftEmail(draftEmail):
            self.title1.text = "Me"
            self.title2.text = draftEmail.displayName
            self.title3.text = ""

        default:
            self.title1.text = " "
            self.title2.text = displayName
            self.title3.text = " "
        }

        self.icon.image = ObjectsIcon.icon(forAnyTrait: priorityTrait, color: UIColor.white, dimension: CGFloat(24))
        self.icon.backgroundColor = Stylesheet.objectColor(forId: priorityTrait.constants?.color ?? 0)
        self.icon.contentMode = UIView.ContentMode.center
        self.icon.layer.cornerRadius = 22
    }
}

class ChildrenViewSwipeAction {
    let icon: FontAwesome
    let handler: (EntityExt) -> Void
    let color: UIColor
    let state: MCSwipeTableViewCellState
    let mode: MCSwipeTableViewCellMode

    init(action: FontAwesome, color: UIColor, state: MCSwipeTableViewCellState, mode: MCSwipeTableViewCellMode = .exit, handler: @escaping (EntityExt) -> Void) {
        self.icon = action
        self.color = color
        self.state = state
        self.mode = mode
        self.handler = handler
    }
}
