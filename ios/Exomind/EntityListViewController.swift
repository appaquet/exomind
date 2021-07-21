import UIKit
import FontAwesome_swift
import Exocore
import SwiftUI

class EntityListViewController: UITableViewController {
    private var query: ExpandableQuery?
    private var parentId: EntityId?

    private var collectionData: [EntityResult] = []
    private var itemClickHandler: ((EntityExt) -> Void)?
    private var collectionClickHandler: ((EntityExt) -> Void)?
    private var swipeActions: [EntityListSwipeAction] = []

    private var switcherButton: SwitcherButton?
    private var switcherButtonActions: [SwitcherButtonAction] = []

    private var scrollEverDragged = false
    private var scrollDragging = false
    private var headerShown: Bool = false
    private var headerWasShownBeforeDrag: Bool = false

    private lazy var datasource: EditableDataSource = {
        var ds = EditableDataSource(tableView: self.tableView) { [weak self] tableView, indexPath, item in
            self?.createCell(indexPath, result: item)
        }

        // all animations are just weird, but there still seems to be some kind of animation that is good enough
        ds.defaultRowAnimation = .none

        return ds
    }()

    override func viewDidLoad() {
        super.viewDidLoad()
        self.tableView.dataSource = self.datasource
        self.tableView.delegate = self

        // enable cell autolayout
        self.tableView.rowHeight = UITableView.automaticDimension
        self.tableView.estimatedRowHeight = 60

        self.tableView.register(EntityListViewCellHost.self, forCellReuseIdentifier: "cell")
    }

    func setSwipeActions(_ actions: [EntityListSwipeAction]) {
        self.swipeActions = actions
    }

    func setClickHandlers(_ itemClick: @escaping (EntityExt) -> Void, collectionClick: @escaping (EntityExt) -> Void) {
        self.itemClickHandler = itemClick
        self.collectionClickHandler = collectionClick
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
            DispatchQueue.main.async { [weak self] in
                self?.setData(newResults)
            }
        }
    }

    private func setData(_ newResults: [EntityResult]) {
        let firstLoad = self.collectionData.isEmpty
        self.collectionData = newResults

        var snapshot = NSDiffableDataSourceSnapshot<Int, EntityResult>()
        snapshot.appendSections([1])
        snapshot.appendItems(newResults)
        self.datasource.apply(snapshot, animatingDifferences: !firstLoad)
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        var top = CGFloat(0.0)
        let bottom = Stylesheet.quickButtonSize + 20

        let hasHeader = self.switcherButtonActions.count > 0
        let showHeader = hasHeader && ((self.scrollDragging && self.headerWasShownBeforeDrag) || (!self.scrollDragging && self.headerShown))
        if (!showHeader) {
            top = top - (self.tableView.tableHeaderView?.frame.height ?? 0)
        }
        let newInsets = UIEdgeInsets(top: top, left: 0, bottom: bottom, right: 0)
        self.tableView.contentInset = newInsets
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
        self.tableView.deselectRow(at: indexPath, animated: false)
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

    override func tableView(_ tableView: UITableView, heightForRowAt indexPath: IndexPath) -> CGFloat {
        let index = (indexPath as NSIndexPath).item
        let entity = self.collectionData[index]

        if entity.collections.isEmpty {
            return 75
        } else {
            return 99
        }
    }

    private func createCell(_ indexPath: IndexPath, result: EntityResult) -> EntityListViewCellHost {
        // TODO: Get reusing back by calculating sizes ahead of time and reuse cells of same height
        //       tableView.dequeueReusableCell(withIdentifier: "cell", for: indexPath) as! EntityListViewCellHost

        let tableViewCell = EntityListViewCellHost()

        let data = cellDataFromResult(result, parentId: self.parentId)
        let view = EntityListViewCell(data: data)

        // From https://stackoverflow.com/questions/59881164/uitableview-with-uiviewrepresentable-in-swiftui
        let controller = UIHostingController(rootView: AnyView(view))
        tableViewCell.host = controller
        let tableCellViewContent = controller.view!
        tableCellViewContent.translatesAutoresizingMaskIntoConstraints = false
        tableViewCell.contentView.addSubview(tableCellViewContent)
        tableCellViewContent.topAnchor.constraint(equalTo: tableViewCell.contentView.topAnchor).isActive = true
        tableCellViewContent.leftAnchor.constraint(equalTo: tableViewCell.contentView.leftAnchor).isActive = true
        tableCellViewContent.bottomAnchor.constraint(equalTo: tableViewCell.contentView.bottomAnchor).isActive = true
        tableCellViewContent.rightAnchor.constraint(equalTo: tableViewCell.contentView.rightAnchor).isActive = true
        tableViewCell.setNeedsLayout()
        tableViewCell.layoutIfNeeded()

        return tableViewCell
    }

    private func swipeActionsForRow(swipeAction: EntityListSwipeAction, indexPath: IndexPath) -> UIContextualAction {
        let index = (indexPath as NSIndexPath).item
        let item = self.collectionData[index]
        let doneAction = UIContextualAction(style: swipeAction.style, title: nil) { action, view, completionHandler in
            swipeAction.handler(item.entity) { [weak self] (completed) in
                guard let this = self else {
                    return
                }

                if completed && swipeAction.style == .destructive,
                   let cutItem = this.collectionData.element(at: index),
                   cutItem.entity.id == item.entity.id {

                    // remove right away from data to make it faster
                    this.collectionData.remove(at: index)
                    this.setData(this.collectionData)
                    completionHandler(true)
                } else {
                    completionHandler(completed)
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

            let onCollectionClick: (EntityExt) -> Void = { [weak self] collection in
                self?.collectionClickHandler?(collection)
            }
            let collections = Collections.instance.entityParentsPillData(entity: entity, onCollectionClick: onCollectionClick)
                    .filter { pillData in
                        // exclude pill if it's from the collection we're showing
                        parentId != pillData.id
                    }

            return EntityResult(result: res, entity: entity, priorityTrait: entity.priorityTrait, collections: collections)
        }
    }

    deinit {
        print("EntityListViewController > Deinit")
    }
}

struct EntityListSwipeAction {
    let icon: FontAwesome
    let handler: Handler
    let color: UIColor
    let side: SwipeSide
    let style: UIContextualAction.Style

    fileprivate let iconImage: UIImage

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

        self.iconImage = ObjectsIcon.icon(forFontAwesome: self.icon, color: UIColor.white, dimension: 30)
    }
}

fileprivate class EditableDataSource: UITableViewDiffableDataSource<Int, EntityResult> {
    override func tableView(_ tableView: UITableView, canEditRowAt indexPath: IndexPath) -> Bool {
        // allows swipe actions on the cells
        // see https://stackoverflow.com/questions/57898044/unable-to-swipe-to-delete-with-tableview-using-diffable-data-source-in-ios-13
        true
    }
}

fileprivate func cellDataFromResult(_ result: EntityResult, parentId: EntityId?) -> EntityListCellData {
    guard let priorityTrait = result.priorityTrait else {
        let img = ObjectsIcon.icon(forFontAwesome: .question, color: .white, dimension: 24)
        return EntityListCellData(image: img, date: result.entity.anyDate, color: UIColor.red, title: "UNKNOWN ENTITY TRAIT")
    }

    let entity = result.entity
    let displayName = priorityTrait.strippedDisplayName
    let image = ObjectsIcon.icon(forAnyTrait: priorityTrait, color: UIColor.white, dimension: CGFloat(24))
    let date = priorityTrait.modificationDate ?? priorityTrait.creationDate
    let color = Stylesheet.objectColor(forId: priorityTrait.constants?.color ?? 0)

    switch priorityTrait.typeInstance() {
    case let .email(email):
        return EntityListCellData(image: image, date: date, color: color, title: Emails.formatContact(email.message.from), subtitle: displayName, text: email.message.snippet, collections: result.collections)

    case let .emailThread(emailThread):
        let emails = entity.traitsOfType(Exomind_Base_Email.self)

        var title = Emails.formatContact(emailThread.message.from)
        if emails.count > 1 {
            title = "\(title) (\(emails.count))"
        }

        if !emailThread.message.read {
            // TODO: Handle this
        }

        let lastEmail = emails.max(by: { (a, b) -> Bool in
            let aDate = a.modificationDate ?? a.creationDate
            let bDate = b.modificationDate ?? b.creationDate
            return aDate < bDate
        })

        var emailDate = date
        if let lastEmail = lastEmail {
            emailDate = lastEmail.modificationDate ?? lastEmail.creationDate
        }

        return EntityListCellData(image: image, date: emailDate, color: color, title: title, subtitle: displayName, text: emailThread.message.snippet, collections: result.collections)

    case let .draftEmail(draftEmail):
        return EntityListCellData(image: image, date: date, color: color, title: "Me", subtitle: draftEmail.displayName, collections: result.collections)

    default:
        return EntityListCellData(image: image, date: date, color: color, title: displayName, collections: result.collections)
    }
}

fileprivate struct EntityResult: Equatable, Hashable {
    let result: Exocore_Store_EntityResult
    let entity: EntityExt
    let priorityTrait: AnyTraitInstance?
    let collections: [CollectionPillData]

    static func ==(lhs: EntityResult, rhs: EntityResult) -> Bool {
        lhs.result == rhs.result
    }

    func hash(into hasher: inout Hasher) {
        self.result.hash(into: &hasher)
    }
}
