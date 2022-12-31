import UIKit
import FontAwesome_swift
import Exocore
import SwiftUI

class EntityListViewController: UITableViewController {
    private var query: ManagedQuery?
    private var parentId: EntityId?

    private var collectionData: [EntityResult] = []

    private var switcherButton: SwitcherButton?
    private var switcherButtonActions: [SwitcherButtonAction] = []

    private var scrollEverDragged = false
    private var scrollDragging = false
    private var headerShown: Bool = false
    private var headerWasShownBeforeDrag: Bool = false
    private var previewedEntity: EntityExt?

    var actionsForEntity: ((EntityExt) -> [Action])?
    var actionsForSelectedEntities: (([EntityExt]) -> [Action])?
    var itemClickHandler: ((EntityExt) -> Void)?
    var collectionClickHandler: ((_ entity: EntityExt, _ collection: EntityExt) -> Void)?

    private lazy var datasource: EditableDataSource = {
        var ds = EditableDataSource(tableView: self.tableView) { [weak self] tableView, indexPath, item in
            self?.createCell(indexPath, result: item)
        }

        // all animations are just weird, but there still seems to be some kind of animation that is good enough
        ds.defaultRowAnimation = .none

        return ds
    }()

    var editMode: Bool {
        set {
            self.tableView.allowsMultipleSelectionDuringEditing = newValue
            self.tableView.setEditing(newValue, animated: true)
            self.setupSelectedItemToolbarActions()
        }
        get {
            self.tableView.isEditing
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.tableView.dataSource = self.datasource
        self.tableView.delegate = self

        self.tableView.register(SwiftUICellViewHost<EntityListCell>.self, forCellReuseIdentifier: "cell")
        self.tableView.rowHeight = UITableView.automaticDimension
        self.tableView.estimatedRowHeight = 75

        NotificationCenter.default.addObserver(self, selector: #selector(onCollectionsChanged), name: .exomindCollectionsChanged, object: nil)
    }

    @objc private func onCollectionsChanged() {
        DispatchQueue.main.async { [weak self] in
            self?.collectionData = []
            self?.refreshData()
        }
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
                .withTrait(Exomind_Base_V1_CollectionChild.self, query: traitQuery)
                .orderByField("weight", ascending: false)
                .project(withProjections: [projectSummaryFields, projectSkipRest])
                .count(30)
                .build()

        self.parentId = entityId
        self.loadData(fromQuery: query)
    }

    func loadData(fromQuery query: Exocore_Store_EntityQuery) {
        print("EntityListViewController > Setting query")
        self.query = ManagedQuery(query: query) { [weak self] in
            self?.refreshData()
        }
    }

    private func refreshData() {
        let newResults = self.convertResults(oldResults: self.collectionData, newResults: self.query?.results ?? [])
        DispatchQueue.main.async { [weak self] in
            self?.setData(newResults)
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
        if self.editMode {
            self.setupSelectedItemToolbarActions()
            return
        }

        self.tableView.deselectRow(at: indexPath, animated: false) // prevent selection highlight
        if let res = self.collectionData.element(at: (indexPath as NSIndexPath).item),
           let handler = self.itemClickHandler {
            handler(res.entity)
        }
    }

    override func tableView(_ tableView: UITableView, didDeselectRowAt indexPath: IndexPath) {
        self.setupSelectedItemToolbarActions()
    }

    override func tableView(_ tableView: UITableView, leadingSwipeActionsConfigurationForRowAt indexPath: IndexPath) -> UISwipeActionsConfiguration? {
        guard let entity = self.collectionData.element(at: (indexPath as NSIndexPath).item)?.entity,
              let actionsForEntity = self.actionsForEntity
        else {
            return nil
        }

        let actions = actionsForEntity(entity)
        let swipeActions: [UIContextualAction] = actions.filter {
                    $0.swipeSide == .leading
                }
                .prefix(2).map {
                    self.toSwipeAction(indexPath: indexPath, action: $0)
                }
        if swipeActions.isEmpty {
            return nil
        }

        return UISwipeActionsConfiguration(actions: swipeActions)
    }

    override func tableView(_ tableView: UITableView, trailingSwipeActionsConfigurationForRowAt indexPath: IndexPath) -> UISwipeActionsConfiguration? {
        guard let entity = self.collectionData.element(at: (indexPath as NSIndexPath).item)?.entity,
              let actionsForEntity = self.actionsForEntity
        else {
            return nil
        }

        let actions = actionsForEntity(entity)
        let swipeActions: [UIContextualAction] = actions.filter {
                    $0.swipeSide == .trailing
                }
                .prefix(2).map {
                    self.toSwipeAction(indexPath: indexPath, action: $0)
                }
        if swipeActions.isEmpty {
            return nil
        }
        return UISwipeActionsConfiguration(actions: swipeActions)
    }

    override func tableView(_ tableView: UITableView, contextMenuConfigurationForRowAt indexPath: IndexPath, point: CGPoint) -> UIContextMenuConfiguration? {
        guard let entity = self.collectionData.element(at: (indexPath as NSIndexPath).item)?.entity,
              let actionsForEntity = self.actionsForEntity
        else {
            return nil
        }

        let actions = actionsForEntity(entity)
        if actions.isEmpty {
            return nil
        }

        let actionProvider: UIContextMenuActionProvider = { _ in
            self.actionsToMenu(actions: actions)
        }

        var previewProvider: UIContextMenuContentPreviewProvider? = nil
        if entity.priorityTrait?.constants?.canPreview ?? false {
            previewProvider = {
                let vc = EntityViewController()
                vc.populate(entity: entity)
                return vc
            }
            previewedEntity = entity
        }

        return UIContextMenuConfiguration(identifier: nil, previewProvider: previewProvider, actionProvider: actionProvider)
    }

    override func tableView(_ tableView: UITableView, willPerformPreviewActionForMenuWith configuration: UIContextMenuConfiguration, animator: UIContextMenuInteractionCommitAnimating) {
        animator.addCompletion {
            if let previewedEntity = self.previewedEntity {
                self.itemClickHandler?(previewedEntity)
            }
        }
    }

    private func createCell(_ indexPath: IndexPath, result: EntityResult) -> SwiftUICellViewHost<EntityListCell> {
        let cell = tableView.dequeueReusableCell(withIdentifier: "cell", for: indexPath) as! SwiftUICellViewHost<EntityListCell>
        let data = cellDataFromResult(result, parentId: self.parentId)
        cell.setView(view: EntityListCell(data: data), parentController: self)
        return cell
    }

    private func toSwipeAction(indexPath: IndexPath, action: Action) -> UIContextualAction {
        let index = (indexPath as NSIndexPath).item

        var style: UIContextualAction.Style = .normal
        if action.destructive {
            style = .destructive
        }

        let sAction = UIContextualAction(style: style, title: nil) { _swipeAction, _view, swipeCb in
            action.execute { [weak self] (result: ActionResult) in
                guard let this = self else {
                    return
                }

                switch result {
                case .successRemoved:
                    DispatchQueue.main.async {
                        // make sure that we don't refresh during an animation of the swipe actions
                        this.query?.inhibitChanges()

                        // remove right away from data to make it faster
                        this.collectionData.remove(at: index)
                        this.setData(this.collectionData)
                        swipeCb(true)
                    }
                case .success:
                    swipeCb(true)
                default:
                    swipeCb(false)
                }
            }
        }
        sAction.backgroundColor = action.swipeColor ?? UIColor.blue
        sAction.image = action.icon.map {
            ObjectsIcon.icon(forFontAwesome: $0, color: UIColor.white, dimension: 30)
        }

        return sAction
    }

    private func showMoreSwipeAction(indexPath: IndexPath, actions: [Action]) -> UIContextualAction {
        let sAction = UIContextualAction(style: .normal, title: "More...") { _swipeAction, view, swipeCb in
            swipeCb(true)

            let alertController = UIAlertController(title: nil, message: nil, preferredStyle: .actionSheet)
            for action in actions {
                let alertBtn = UIAlertAction(title: action.label, style: .default) { alertAction in
                    action.execute({ res in })
                }

                if let icon = action.icon {
                    let img = ObjectsIcon.icon(forFontAwesome: icon, color: UIColor.white, dimension: 30)
                    alertBtn.setValue(img, forKey: "image")
                }

                alertController.addAction(alertBtn)
            }
            alertController.addAction(UIAlertAction(title: "Cancel", style: .cancel))
            self.present(alertController, animated: true)
        }

        sAction.backgroundColor = Stylesheet.collectionSwipeMoreBg
        sAction.image = ObjectsIcon.icon(forFontAwesome: .ellipsisH, color: UIColor.white, dimension: 30)
        return sAction
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
                self?.collectionClickHandler?(entity, collection)
            }
            let collections = Collections.instance.entityParentsPillData(entity: entity, onCollectionClick: onCollectionClick)
                    .filter { pillData in
                        // exclude pill if it's from the collection we're showing
                        parentId != pillData.id
                    }

            return EntityResult(result: res, entity: entity, priorityTrait: entity.priorityTrait, collections: collections)
        }
    }

    private func setupSelectedItemToolbarActions() {
        if !self.editMode {
            self.navigationController?.setToolbarHidden(true, animated: true)
            return
        }

        let selectedPaths = self.tableView.indexPathsForSelectedRows ?? []
        let selectedEntities = selectedPaths.map {
            self.collectionData[$0.row].entity
        }
        if selectedEntities.isEmpty {
            self.navigationController?.setToolbarHidden(true, animated: true)
            return
        }

        let actions = self.actionsForSelectedEntities?(selectedEntities) ?? []
        if actions.isEmpty {
            self.navigationController?.setToolbarHidden(true, animated: true)
            return
        }

        var items = [UIBarButtonItem]()
        items.append(UIBarButtonItem(barButtonSystemItem: .flexibleSpace, target: nil, action: nil))

        for action in actions.prefix(3) {
            guard let icon = action.icon else {
                continue
            }

            let img = ObjectsIcon.icon(forFontAwesome: icon, color: UIColor.label, dimension: 30)
            items.append(UIBarButtonItem(image: img, primaryAction: UIAction { _ in
                action.execute { _ in
                }
                self.editMode = false
            }))
            items.append(UIBarButtonItem(barButtonSystemItem: .flexibleSpace, target: nil, action: nil))
        }

        if actions.count > 3 {
            let img = ObjectsIcon.icon(forFontAwesome: .ellipsisH, color: UIColor.label, dimension: 30)
            let menu = self.actionsToMenu(actions: actions) { (_, _) in
                self.editMode = false
            }
            items.append(UIBarButtonItem(image: img, menu: menu))
        }

        items.append(UIBarButtonItem(barButtonSystemItem: .flexibleSpace, target: nil, action: nil))

        self.topMostParent()?.toolbarItems = items
        self.navigationController?.setToolbarHidden(false, animated: true)
    }

    // Find parent in hierarchy that is right under the navigation controller.
    // This is in this last parent that the toolbar items can be added to.
    private func topMostParent() -> UIViewController? {
        var current: UIViewController? = self
        while current?.parent as? UINavigationController == nil {
            current = current?.parent
        }
        return current
    }

    private func actionsToMenu(actions: [Action], cb: ((Action, ActionResult) -> Void)? = nil) -> UIMenu {
        let children: [UIAction] = actions.map { action in
            let image = action.icon.map {
                ObjectsIcon.icon(forFontAwesome: $0, color: UIColor.label, dimension: 30)
            }

            return UIAction(title: action.label, image: image) { _ in
                action.execute { res in
                    // we delay incoming changes from server to prevent weird animation glitches
                    self.query?.inhibitChanges(forDelay: 0.7)

                    cb?(action, res)
                }
            }
        }

        return UIMenu(children: children)
    }

    deinit {
        print("EntityListViewController > Deinit")
        NotificationCenter.default.removeObserver(self)
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

    var data: EntityListCellData
    switch priorityTrait.typeInstance() {
    case let .email(email):
        data = EntityListCellData(
                image: image,
                date: date,
                color: color,
                title: Emails.formatContact(email.message.from),
                subtitle: displayName,
                text: email.message.snippet
        )

    case let .emailThread(emailThread):
        let emails = entity.traitsOfType(Exomind_Base_V1_Email.self)

        var title = Emails.formatContact(emailThread.message.from)
        if emails.count > 1 {
            title = "\(title) (\(emails.count))"
        }

        let unread = !entity.traitsOfType(Exomind_Base_V1_Unread.self).isEmpty

        let lastEmail = emails.max(by: { (a, b) -> Bool in
            let aDate = a.modificationDate ?? a.creationDate
            let bDate = b.modificationDate ?? b.creationDate
            return aDate < bDate
        })

        var emailDate = date
        if let lastEmail = lastEmail {
            emailDate = lastEmail.modificationDate ?? lastEmail.creationDate
        }

        data = EntityListCellData(
                image: image,
                date: emailDate,
                color: color,
                title: title,
                subtitle: displayName,
                text: emailThread.message.snippet
        ).withBold(enabled: unread)

    case let .draftEmail(draftEmail):
        data = EntityListCellData(
                image: image,
                date: date,
                color: color,
                title: "Me",
                subtitle: draftEmail.displayName
        )

    case let .collection(collection):
        if !collection.message.description_p.isEmpty {
            data = EntityListCellData(
                    image: image,
                    date: date,
                    color: color,
                    title: collection.strippedDisplayName,
                    text: collection.message.description_p
            )
        } else {
            data = EntityListCellData(
                    image: image,
                    date: date,
                    color: color,
                    title: collection.strippedDisplayName
            )
        }

    default:
        data = EntityListCellData(
                image: image,
                date: date,
                color: color,
                title: displayName
        )
    }

    if !result.collections.isEmpty {
        data = data.withCollections(result.collections)
    }

    var indicators: [UIImage] = []
    if isSnoozed(result) {
        indicators.append(ObjectsIcon.icon(forFontAwesome: .clock, color: .lightGray, dimension: 16))
    }

    if let parentId = parentId, Collections.isPinnedInParent(result.entity, parentId: parentId) {
        indicators.append(ObjectsIcon.icon(forFontAwesome: .thumbtack, color: .lightGray, dimension: 16))
    }

    if !indicators.isEmpty {
        data = data.withIndicators(indicators)
    }

    return data
}

fileprivate func isSnoozed(_ result: EntityResult) -> Bool {
    result.entity.traitOfType(Exomind_Base_V1_Snoozed.self) != nil
}

fileprivate struct EntityResult: Equatable, Hashable {
    let result: Exocore_Store_EntityResult
    let entity: EntityExt
    let priorityTrait: AnyTraitInstance?
    let collections: [CollectionPillData]

    static func ==(lhs: EntityResult, rhs: EntityResult) -> Bool {
        lhs.result == rhs.result && lhs.collections == rhs.collections
    }

    func hash(into hasher: inout Hasher) {
        self.result.hash(into: &hasher)
        for collection in collections {
            collection.id.hash(into: &hasher)
        }
    }
}
