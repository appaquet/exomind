import UIKit
import FontAwesome_swift
import Dwifft
import Exocore

class EntityListViewController: UITableViewController {
    private var query: ExpandableQuery?
    private var parentId: EntityId?

    private var collectionData: [EntityResult] = []
    private var itemClickHandler: ((EntityExt) -> Void)?
    private var swipeActions: [ChildrenViewSwipeAction] = []

    private var switcherButton: SwitcherButton?
    private var switcherButtonActions: [SwitcherButtonAction] = []

    private var scrollEverDragged = false
    private var scrollDragging = false
    private var headerShown: Bool = false
    private var headerWasShownBeforeDrag: Bool = false

    private var diffCalculator: SingleSectionTableViewDiffCalculator<EntityResult>?

    override func viewDidLoad() {
        super.viewDidLoad()
        self.tableView.delegate = self
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

    func loadData(withResults results: [Exocore_Store_EntityResult]) {
        self.collectionData = convertResults(oldResults: self.collectionData, newResults: results)
    }

    func loadData(fromChildrenOf entityId: EntityId) {
        let traitQuery = TraitQueryBuilder.refersTo(field: "collection", entityId: entityId).build()

        var projectSummaryFields = Exocore_Store_Projection()
        projectSummaryFields.fieldGroupIds = [1]
        projectSummaryFields.package = ["exomind.base"]
        var projectSkipRest = Exocore_Store_Projection()
        projectSkipRest.skip = true

        let query = QueryBuilder
                .withTrait(Exomind_Base_CollectionChild.self, query: traitQuery)
                .orderByField("weight", ascending: false)
                .project(withProjections: [projectSummaryFields, projectSkipRest])
                .count(30)
                .build()

        self.parentId = entityId
        self.loadData(fromQuery: query)
    }

    func loadData(fromQuery query: Exocore_Store_EntityQuery) {
        self.query = ExpandableQuery(query: query) { [weak self] in
            guard let this = self else {
                return
            }

            let newResults = this.convertResults(oldResults: this.collectionData, newResults: this.query?.results ?? [])
            DispatchQueue.main.async {
                // on first load, we load data directly to prevent diff & animation
                if this.collectionData.isEmpty && !newResults.isEmpty {
                    this.collectionData = newResults
                    this.tableView.reloadData()

                    this.diffCalculator = SingleSectionTableViewDiffCalculator<EntityResult>(tableView: this.tableView, initialRows: this.collectionData)
                    this.diffCalculator?.insertionAnimation = .fade
                    this.diffCalculator?.deletionAnimation = .fade
                } else {
                    this.collectionData = newResults
                    this.diffCalculator?.rows = this.collectionData
                }
            }
        }
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        var top = CGFloat(0.0)
        let bottom = Stylesheet.quickButtonSize + 20
        let showHeader = self.hasHeader && ((self.scrollDragging && self.headerWasShownBeforeDrag) || (!self.scrollDragging && self.headerShown))
        if (!showHeader) {
            top = top - (self.tableView.tableHeaderView?.frame.height ?? 0)
        }
        let newInsets = UIEdgeInsets(top: top, left: 0, bottom: bottom, right: 0)
        self.tableView.contentInset = newInsets
    }

    private var hasHeader: Bool {
        get {
            self.switcherButtonActions.count > 0
        }
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        self.collectionData.count
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = self.tableView.dequeueReusableCell(withIdentifier: "cell", for: indexPath) as! ChildrenViewCell

        // fixes borders between each cell that otherwise aren't 100% width
        cell.layoutMargins = UIEdgeInsets.zero
        cell.preservesSuperviewLayoutMargins = false

        cell.populate(&self.collectionData[(indexPath as NSIndexPath).item])

        return cell
    }

    override func scrollViewDidScroll(_ scrollView: UIScrollView) {
        let headerHeight = (self.tableView.tableHeaderView?.frame.height ?? CGFloat(44))

        let totalHeight = (scrollView.contentSize.height - tableView.frame.size.height)
        let currentPosition = scrollView.contentOffset.y
        let averageHeight = CGFloat(74)
        let itemsComingUp = (totalHeight - currentPosition) / averageHeight

        let canExpand = self.query?.canExpand ?? false
        if (itemsComingUp < 10 && canExpand) {
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
            handler(res.entity)
        }
    }

    override func tableView(_ tableView: UITableView, leadingSwipeActionsConfigurationForRowAt indexPath: IndexPath) -> UISwipeActionsConfiguration? {
        let swipeActions = self.swipeActions
                .filter({ $0.side == .leading })
                .map({ self.swipeActionsForRow(swipeAction: $0, indexPath: indexPath) })
        return UISwipeActionsConfiguration(actions: swipeActions)
    }

    override func tableView(_ tableView: UITableView, trailingSwipeActionsConfigurationForRowAt indexPath: IndexPath) -> UISwipeActionsConfiguration? {
        let swipeActions = self.swipeActions
                .filter({ $0.side == .trailing })
                .map({ self.swipeActionsForRow(swipeAction: $0, indexPath: indexPath) })
        return UISwipeActionsConfiguration(actions: swipeActions)
    }

    private func swipeActionsForRow(swipeAction: ChildrenViewSwipeAction, indexPath: IndexPath) -> UIContextualAction {
        let index = (indexPath as NSIndexPath).item
        let item = self.collectionData[index]
        let doneAction = UIContextualAction(style: swipeAction.style, title: nil) { action, view, completionHandler in
            swipeAction.handler(item.entity) { [weak self] (completed) in
                // TODO: Put back when Dwift is removed
//                if completed && swipeAction.style == .destructive,
//                   let cutItem = self?.collectionData.element(at: index),
//                   cutItem.entity.id == item.entity.id {
//
//                    completionHandler(completed)
//                    self?.collectionData.remove(at: index)
//                    self?.tableView.deleteRows(at: [indexPath], with: .automatic)
//                } else {
//                    completionHandler(completed)
//                }

                if completed && swipeAction.style == .destructive {
                    // iOS acts weird here when you have a successful destructive call
                    // the animation seems to be broken (see https://stackoverflow.com/questions/46714155/uitableview-destructive-uicontextualaction-does-not-reload-data)
                    // not calling the completion handler seems to have the best outcome
                } else {
                    completionHandler(true)
                }
            }
        }
        doneAction.backgroundColor = swipeAction.color
        doneAction.image = swipeAction.iconImage
        return doneAction
    }


    private func convertResults(oldResults: [EntityResult], newResults: [Exocore_Store_EntityResult]) -> [EntityResult] {
        var currentResults = [String: EntityResult]()
        for currentResult in oldResults {
            currentResults[currentResult.result.entity.id] = currentResult
        }

        return newResults.map { (res: Exocore_Store_EntityResult) in
            let entityId = res.entity.id
            let entity = res.entity.toExtension()

            if let current = currentResults[entityId], current.entity.anyDate == entity.anyDate {
                return current
            }

            return EntityResult(result: res, entity: entity, priorityTrait: entity.priorityTrait)
        }
    }

    deinit {
        print("EntityViewController > Deinit")
    }
}

extension Array {
    func element(at index: Int) -> Element? {
        index >= 0 && index < count ? self[index] : nil
    }
}


class ChildrenViewCell: UITableViewCell {
    @IBOutlet weak var title1: UILabel!
    @IBOutlet weak var title2: UILabel!
    @IBOutlet weak var title3: UILabel!
    @IBOutlet weak var date: UILabel!
    @IBOutlet weak var icon: UIImageView!

    var entity: EntityExt!

    fileprivate func populate(_ result: inout EntityResult) {
        self.backgroundColor = UIColor.clear

        self.entity = result.entity

        guard let priorityTrait = result.priorityTrait else {
            self.title1.text = "UNKNOWN ENTITY TRAIT"
            self.title2.text = ""
            self.title3.text = ""
            self.date.text = ""
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
    let handler: Handler
    let color: UIColor
    let side: SwipeSide
    let style: UIContextualAction.Style

    enum SwipeSide {
        case leading
        case trailing
    }

    typealias Handler = (EntityExt, @escaping (Bool) -> Void) -> Void

    init(action: FontAwesome, color: UIColor, side: SwipeSide, style: UIContextualAction.Style, handler: @escaping Handler) {
        self.icon = action
        self.color = color
        self.side = side
        self.style = style
        self.handler = handler
    }

    lazy var iconImage = {
        ObjectsIcon.icon(forFontAwesome: self.icon, color: UIColor.white, dimension: 30)
    }()
}


fileprivate struct EntityResult: Equatable {
    var result: Exocore_Store_EntityResult
    var entity: EntityExt
    var priorityTrait: AnyTraitInstance?

    static func ==(lhs: EntityResult, rhs: EntityResult) -> Bool {
        lhs.entity == rhs.entity
    }
}

