//
//  SearchViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-01-06.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit

class SearchViewController: NavigationController {
    var fromObjectId: String?
    var selectionHandler: ((HCEntity) -> Void)?
    
    fileprivate var doneButton: UIBarButtonItem!
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        // set colors of navigation bar
        Stylesheet.styleNavigationBar(self.navigationBar, bgColor: Stylesheet.searchNavigationBarBg, fgColor: Stylesheet.searchNavigationBarFg)
        
        let containerVc = (self.topViewController as! SearchCollectionContainer)
        containerVc.fromObjectId = self.fromObjectId
        containerVc.selectionHandler = self.selectionHandler
        containerVc.searchNavigationController = self
        
        // keep the done button to add it to other views
        self.doneButton = self.topViewController?.navigationItem.rightBarButtonItem
    }
    
    func addDoneButton() {
        self.topViewController?.navigationItem.rightBarButtonItem = doneButton
    }
    
    override func setBarActions(_ actions: [NavigationControllerBarAction]) {
        // we don't allow any custom navigation buttons. Only the done button is added
    }
    
    override func resetState() {
        super.resetState()
        
        // we always add the done button
        self.addDoneButton()
    }
    
    deinit {
        print("SearchViewController > Deinit")
    }
}

class SearchCollectionContainer: UIViewController, UISearchBarDelegate {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    fileprivate var searchBar: UISearchBar!
    fileprivate var childrenViewController: ChildrenViewController!
    fileprivate var fromObjectId: String?
    fileprivate var selectionHandler: ((HCEntity) -> Void)?
    fileprivate weak var searchNavigationController: SearchViewController!
    fileprivate var searchText: String?
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        self.setupChildrenViewController()
        self.searchBar = UISearchBar()
        self.navigationItem.titleView = self.searchBar
        self.searchBar.becomeFirstResponder()
        self.searchBar.placeholder = "Search"
        self.searchBar.delegate = self
    }
    
    fileprivate func setupChildrenViewController() {
        self.childrenViewController = (self.mainStoryboard.instantiateViewController(withIdentifier: "ChildrenViewController") as! ChildrenViewController)
        self.childrenViewController.tableView.keyboardDismissMode = .onDrag
        self.addChild(self.childrenViewController)
        self.view.addSubview(self.childrenViewController.view)
        
        self.childrenViewController.setCollectionQueryBuilder { () -> Query in
            return Query.unitQuery()
        }
        self.childrenViewController.setItemClickHandler { [weak self] (entity) -> Void in
            self?.handleItemSelection(entity)
        }
        self.childrenViewController.loadData(true)
        self.childrenViewController.setSwipeActions([
            ChildrenViewSwipeAction(action: .inbox, color: Stylesheet.collectionSwipeDoneBg, state: .state1, mode: .exit, handler: { [weak self] (entity) -> Void in
                self?.handleCopyInbox(entity)
            })
        ])
    }
    
    override func viewWillAppear(_ animated: Bool) {
        (self.navigationController as? NavigationController)?.resetState()
    }
    
    var debouncedSearch: (()->())?
    func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
        self.searchText = searchText
        
        if self.debouncedSearch == nil {
            self.debouncedSearch = Debouncer.debounce(delay: 500, queue: DispatchQueue.main, action: { [weak self] in
                guard let this = self,
                    let searchText = this.searchText else { return }
                
                this.childrenViewController.setCollectionQueryBuilder { () -> Query? in
                    if (!searchText.isEmpty) {
                        return HCQueries.Entities().matches(query: searchText).withSummary().toDomainQuery()
                    } else {
                        return nil
                    }
                }
                this.childrenViewController.loadData(true)
            })
        }
        self.debouncedSearch?()
    }
    
    fileprivate func handleItemSelection(_ entity: HCEntity) {
        self.searchBar.resignFirstResponder() // prevent keyboard from transitioning weirdly
        if let handler = self.selectionHandler {
            handler(entity)
        } else {
            self.searchNavigationController.pushObject(.entity(entity: entity))
        }
    }
    
    fileprivate func handleCopyInbox(_ entity: HCEntity) {
        ExomindDSL.on(entity).relations.addParent(parentId: "inbox")
    }
    
    override func viewDidAppear(_ animated: Bool) {
        self.searchBar.becomeFirstResponder()
    }
    
    @IBAction func handleDoneClick(_ sender: AnyObject) {
        self.dismiss(animated: true, completion: nil)
    }
    
    deinit {
        print("SearchCollectionContainer > Deinit")
    }
}
