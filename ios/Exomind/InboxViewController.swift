//
//  InboxViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-11-16.
//  Copyright © 2015 Exomind. All rights reserved.
//

import UIKit
import Exocore

class InboxViewController: UIViewController {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)
    fileprivate let entityId: String = "inbox"
    
    fileprivate var childrenViewController: ChildrenViewController!
    fileprivate var childrenType: String!
    fileprivate var inboxEntity: HCEntity!
    
    override func viewDidLoad() {
        super.viewDidLoad()
    
        self.title = "Inbox"
        if let inbox = SessionStore.inboxEntity() {
            self.inboxEntity = inbox
        }
        
        self.setupChildrenViewController()
    }
    
    fileprivate func setupChildrenViewController() {
        self.childrenViewController = (mainStoryboard.instantiateViewController(withIdentifier: "ChildrenViewController") as! ChildrenViewController)
        self.childrenViewController.setParent(withId: "inbox")
        self.childrenViewController.setItemClickHandler { [weak self] in
            self?.handleItemClick($0)
        }
        self.addChild(self.childrenViewController)
        self.view.addSubview(self.childrenViewController.view)
        
        self.switchChildrenType("current")
    }
    
    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        self.setupNavigationActions()
    }
    
    fileprivate func switchChildrenType(_ type: String) {
        self.childrenType = type
        self.setupSwipe()
        self.setupSwitcher()
        
        self.childrenViewController.loadData(true)
        
        self.changeTheme()
        self.setupNavigationActions()
    }
    
    fileprivate func setupNavigationActions() {
        let nav = (self.navigationController as! NavigationController)
        nav.resetState()
        
        nav.setBarActions([
            NavigationControllerBarAction(icon: .search, handler: { [weak self] () -> Void in
                (self?.navigationController as? NavigationController)?.showSearch("inbox")
            })
        ])
        
        // quick button only visible in current
        if (self.childrenType == "current") {
            nav.setQuickButtonActions([
                QuickButtonAction(icon: .clock, handler: { () -> Void in
                }),
                QuickButtonAction(icon: .plus, handler: { [weak self] () -> Void in
                    (self?.navigationController as? NavigationController)?.showCreateObject("inbox") { [weak self] (entity) -> Void in
                        guard let entity = entity else { return }
                        (self?.navigationController as? NavigationController)?.pushObject(.entityOld(entity: entity))
                    }
                }),
                QuickButtonAction(icon: .check, handler: { () -> Void in
                })
            ])
        }
    }
    
    fileprivate func setupSwitcher() {
        self.childrenViewController.setSwitcherActions(
            [
                SwitcherButtonAction(icon: .clock, active: self.childrenType == "future", callback: { [weak self] in
                    self?.switchChildrenType("future")
                }),
                SwitcherButtonAction(icon: .inbox, active: self.childrenType == "current", callback: { [weak self] in
                    self?.switchChildrenType("current")
                }),
                SwitcherButtonAction(icon: .check, active: self.childrenType == "old", callback: { [weak self] in
                    self?.switchChildrenType("old")
                }),
            ])
    }
    
    fileprivate func setupSwipe() {
        if (self.childrenType == "current") {
            self.setupSwipeCurrent()
        } else if (self.childrenType == "old") {
            self.setupSwipeOld()
        } else if (self.childrenType == "future") {
            self.setupSwipeFuture()
        }
    }
    
    fileprivate func setupSwipeCurrent() {
        self.childrenViewController.setSwipeActions(
            [
                ChildrenViewSwipeAction(action: .check, color: Stylesheet.collectionSwipeDoneBg, state: .state1, mode: .exit, handler: { [weak self] (entity) -> Void in
                    self?.handleDone(entity)
                }),
                ChildrenViewSwipeAction(action: .clock, color: Stylesheet.collectionSwipeLaterBg, state: .state3, mode: .switch, handler: { [weak self] (entity) -> Void in
                    self?.handleMoveLater(entity)
                }),
                ChildrenViewSwipeAction(action: .folderOpen, color: Stylesheet.collectionSwipeAddCollectionBg, state: .state4, mode: .switch, handler: { [weak self] (entity) -> Void in
                    self?.handleAddToCollection(entity)
                })
            ]
        )
    }
    
    fileprivate func setupSwipeOld() {
        self.childrenViewController.setSwipeActions(
            [
                ChildrenViewSwipeAction(action: .inbox, color: Stylesheet.collectionSwipeDoneBg, state: .state3, mode: .exit, handler: { [weak self] (entity) -> Void in
                    self?.handleMoveCurrent(entity)
                }),
                ChildrenViewSwipeAction(action: .folderOpen, color: Stylesheet.collectionSwipeAddCollectionBg, state: .state4, mode: .switch, handler: { [weak self] (entity) -> Void in
                    self?.handleAddToCollection(entity)
                })
            ])
    }
    
    fileprivate func setupSwipeFuture() {
        self.childrenViewController.setSwipeActions(
            [
                ChildrenViewSwipeAction(action: .inbox, color: Stylesheet.collectionSwipeDoneBg, state: .state1, mode: .exit, handler: { [weak self] (entity) -> Void in
                    self?.handleRemovePostponed(entity)
                    self?.handleMoveCurrent(entity)
                }),
                ChildrenViewSwipeAction(action: .check, color: Stylesheet.collectionSwipeDoneBg, state: .state2, mode: .exit, handler: { [weak self] (entity) -> Void in
                    self?.handleRemovePostponed(entity)
                    self?.handleDone(entity)
                }),
                ChildrenViewSwipeAction(action: .clock, color: Stylesheet.collectionSwipeLaterBg, state: .state3, mode: .switch, handler: { [weak self] (entity) -> Void in
                    self?.handleMoveLater(entity)
                }),
                ChildrenViewSwipeAction(action: .folderOpen, color: Stylesheet.collectionSwipeAddCollectionBg, state: .state4, mode: .switch, handler: { [weak self] (entity) -> Void in
                    self?.handleAddToCollection(entity)
                })
            ])
    }
    
    fileprivate func changeTheme() {
        if (self.childrenType == "old") {
            self.childrenViewController.setTheme(Stylesheet.collectionSwipeDoneBg.withAlphaComponent(0.3))
        } else if (self.childrenType == "future") {
            self.childrenViewController.setTheme(Stylesheet.collectionSwipeLaterBg.withAlphaComponent(0.3))
        } else {
            self.childrenViewController.setTheme(nil)
        }
    }
    
    fileprivate func handleItemClick(_ entity: EntityExt) {
        (self.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
    }
    
    fileprivate func handleRemovePostponed(_ entity: EntityExt) {
        // TODO:
//        ExomindDSL.on(entity).relations.removePostpone()
    }
    
    fileprivate func handleDone(_ entity: EntityExt) {
        // TODO:
//        ExomindDSL.on(entity).relations.removeParent(parentId: self.entityId)
    }
    
    fileprivate func handleMoveCurrent(_ entity: EntityExt) {
        // TODO:
//        ExomindDSL.on(entity).relations.addParent(parentId: self.entityId)
    }
    
    fileprivate func handleMoveLater(_ entity: EntityExt) {
        // TODO:
//        (self.navigationController as? NavigationController)?.showTimeSelector(forEntity: entity) { completed in
//            if (completed) {
//                ExomindDSL.on(entity).relations.removeParent(parentId: self.entityId)
//            }
//        }
    }
    
    fileprivate func handleAddToCollection(_ entity: EntityExt) {
        // TODO:
//        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: entity)
    }
    
    deinit {
        print("InboxViewController > Deinit")
    }
    
}
