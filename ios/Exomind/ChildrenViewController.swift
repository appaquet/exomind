//
//  ChildrenViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-10-07.
//  Copyright © 2015 Exomind. All rights reserved.
//

import UIKit
import FontAwesome_swift
import MCSwipeTableViewCell
import Dwifft

class ChildrenViewController: UITableViewController {
    fileprivate var childrenType: String = "current"
    
    fileprivate var querySet: QuerySet!
    fileprivate var collectionQuery: Query!
    fileprivate var collectionData: [HCEntity]!
    
    fileprivate var collectionQueryBuilder: (() -> Query?)!
    fileprivate var itemClickHandler: ((HCEntity) -> Void)?
    fileprivate var swipeActions: [ChildrenViewSwipeAction] = []
    
    fileprivate var switcherButton: SwitcherButton?
    fileprivate var switcherButtonActions: [SwitcherButtonAction] = []
    
    fileprivate var scrollEverDragged = false
    fileprivate var scrollDragging = false
    fileprivate var headerShown: Bool = false
    fileprivate var headerWasShownBeforeDrag: Bool = false
    fileprivate var expandPending = false
    
    var diffCalculator: SingleSectionTableViewDiffCalculator<HCEntity>?
    
    override func viewDidLoad() {
        super.viewDidLoad()
        self.tableView.delegate = self
        self.diffCalculator = SingleSectionTableViewDiffCalculator<HCEntity>(tableView: self.tableView, initialRows: [])
        self.diffCalculator?.insertionAnimation = .fade
        self.diffCalculator?.deletionAnimation = .fade
    }
    
    func setSwipeActions(_ actions: [ChildrenViewSwipeAction]) {
        self.swipeActions = actions
    }
    
    func setCollectionQueryBuilder(_ builder: @escaping () -> Query?) {
        self.collectionQueryBuilder = builder
    }
    
    func setItemClickHandler(_ handler: @escaping (HCEntity) -> Void) {
        self.itemClickHandler = handler
    }
    
    func isChildrenCurrent() -> Bool {
        return self.childrenType == "current"
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
    
    func loadData(_ reExecute: Bool = false) {
        if (self.querySet == nil) {
            self.querySet = DomainStore.instance.getQuerySet()
            self.querySet.onChange { [weak self] () -> () in
                if  let this = self,
                    let collectionQuery = this.collectionQuery,
                    collectionQuery.hasResults() {

                    // conversion to entities is expensive, we push it to background and then render on main
                    DispatchQueue.global(qos: .background).async {
                        this.collectionData = this.collectionQuery.resultsAsEntities()
                        DispatchQueue.main.async {
                            this.diffCalculator?.rows = this.collectionData
                        }
                    }
                }
            }
        }
        
        if let builder = self.collectionQueryBuilder, let query = builder() {
            self.collectionQuery = self.querySet.executeQuery(query, reExecute: reExecute)
        }
    }
    
    func setTheme(_ color: UIColor?) {
        if let color = color {
            let bgView = UIView()
            bgView.frame = self.tableView.frame
            bgView.backgroundColor = color
            self.tableView.backgroundView = bgView
        } else {
            self.tableView.backgroundView = nil
        }
    }
    
    func hasHeader() -> Bool {
        return self.switcherButtonActions.count > 0
    }
    
    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        var top = CGFloat(0.0)
        let bottom = Stylesheet.quickButtonSize + 20
        let showHeader = self.hasHeader() && (!self.isChildrenCurrent() || (scrollDragging && headerWasShownBeforeDrag) || (!scrollDragging && headerShown))
        if (!showHeader) {
            top = top - (self.tableView.tableHeaderView?.frame.height ?? 0)
        }
        let newInsets = UIEdgeInsets(top: top, left: 0, bottom: bottom, right: 0)
        self.tableView.contentInset = newInsets
    }
    
    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        return self.collectionData?.count ?? 0
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
            self.expandPending = false
        }
        
        // if only 5 items or less are coming up, we load new
        if (itemsComingUp < 10 && self.collectionQuery.isLoaded() && !self.expandPending) {
            self.expandPending = true
            DispatchQueue.main.async(execute: { [weak self] () -> Void in
                if let this = self, let query = this.collectionQuery.expand() {
                    print("ChildrenViewController > Expanding query...")
                    this.collectionQuery = this.querySet.executeQuery(query, reExecute: true)
                }
            })
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
        let entity = self.collectionData[(indexPath as NSIndexPath).item]
        if let handler = self.itemClickHandler {
            handler(entity)
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
            cell.setSwipeGestureWith(swipeIconView, color: action.color, mode: action.mode, state: action.state) {
                (scell, state, mode) -> Void in
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
    
    var entity: HCEntity!
    
    func populate(_ entity: HCEntity) {
        self.backgroundColor = UIColor.clear
        self.entity = entity
        
        guard let entityTrait = EntityTrait(entity: entity)
            else {
                self.title1.text = "UNKNOWN ENTITY TRAIT"
                return
            }
        
        let displayName = entityTrait.displayName
        self.date.text = entityTrait.trait.modificationDate?.toShort() ?? entityTrait.trait.creationDate?.toShort() ?? ""
        
        
        self.title1.font = UIFont.systemFont(ofSize: 14)
        self.title2.font = UIFont.systemFont(ofSize: 14)
        self.title3.font = UIFont.systemFont(ofSize: 14)
        switch (entityTrait.traitType) {
        case let .email(email: email):
            self.title1.text = EmailsLogic.formatContact(email.from)
            self.title2.text = displayName
            self.title3.text = email.snippet
            
        case let .emailThread(emailThread: emailThread):
            let emails = entity.traitsByType[EmailSchema.fullType]
            
            self.title1.text = EmailsLogic.formatContact(emailThread.from)
            if let emails = emails {
                if emails.count > 1 {
                    self.title1.text = "\(self.title1.text!) (\(emails.count))"
                }
                
                if !emails.filter({ ($0 as? Email)?.unread ?? true }).isEmpty {
                    self.title1.font = UIFont.boldSystemFont(ofSize: 14)
                    self.title2.font = UIFont.boldSystemFont(ofSize: 14)
                    self.title3.font = UIFont.boldSystemFont(ofSize: 14)
                }
                
                let lastEmail = emails.max(by: { (a, b) -> Bool in
                    let aDate = a.modificationDate ?? a.creationDate ?? Date()
                    let bDate = b.modificationDate ?? b.creationDate ?? Date()
                    return aDate < bDate
                })
                if let lastEmail = lastEmail, let date = lastEmail.modificationDate ?? lastEmail.creationDate {
                    self.date.text = date.toShort()
                }
            }
            self.title2.text = displayName
            self.title3.text = emailThread.snippet
            
        case let .draftEmail(draftEmail: draftEmail):
            self.title1.text = "Me"
            self.title2.text = draftEmail.subject ?? "Untitled email"
            self.title3.text = ""
            
        default:
            self.title1.text = " "
            self.title2.text = displayName
            self.title3.text = " "
        }
        
        self.icon.image = ObjectsIcon.icon(forEntityTrait: entityTrait, color: UIColor.white, dimension: CGFloat(24))
        self.icon.backgroundColor = Stylesheet.objectColor(forId: entityTrait.color)
        self.icon.contentMode = UIView.ContentMode.center
        self.icon.layer.cornerRadius = 22
    }
}

class ChildrenViewSwipeAction {
    let icon: FontAwesome
    let handler: (HCEntity) -> Void
    let color: UIColor
    let state: MCSwipeTableViewCellState
    let mode: MCSwipeTableViewCellMode
    
    init(action: FontAwesome, color: UIColor, state: MCSwipeTableViewCellState, mode: MCSwipeTableViewCellMode = .exit, handler: @escaping (HCEntity) -> Void) {
        self.icon = action
        self.color = color
        self.state = state
        self.mode = mode
        self.handler = handler
    }
}
