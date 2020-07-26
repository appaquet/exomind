//
//  CollectionViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-12-08.
//  Copyright © 2015 Exomind. All rights reserved.
//

import UIKit

class CollectionViewController: UIViewController, EntityTraitView {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    fileprivate var childrenType: String = "current"
    fileprivate var entity: EntityExt!
    fileprivate var trait: AnyTraitInstance!
    fileprivate var childrenViewController: ChildrenViewController!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance) {
        self.entity = entity
        self.trait = trait
        self.loadData()
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        self.setupChildrenViewController()
        self.loadData()
    }

    fileprivate func setupChildrenViewController() {
        self.childrenViewController = (mainStoryboard.instantiateViewController(withIdentifier: "ChildrenViewController") as! ChildrenViewController)
        self.childrenViewController.setParent(withId: self.entity.id)
        self.childrenViewController.setItemClickHandler { [weak self] in
            self?.handleItemClick($0)
        }
//        self.childrenViewController.setCollectionQueryBuilder { [weak self] () -> Query? in
//            guard let this = self else {
//                return nil
//            }
//
//            if this.childrenType == "current" {
//                return HCQueries.Entities().withParent(entityId: this.entityId).withSummary().toDomainQuery()
//            } else if this.childrenType == "old" {
//                return HCQueries.Entities().withTrait(OldChildSchema.fullType, traitBuilder: { (q) in
//                    q.refersTo(this.entityId)
//                }).withSummary().toDomainQuery()
//            } else {
//                return nil
//            }
//        }
        self.switchChildrenType("current")
        self.addChild(self.childrenViewController)
        self.view.addSubview(self.childrenViewController.view)
    }

    fileprivate func loadData() {
        if let entityTrait = self.trait {
            self.title = entityTrait.displayName
        }
    }

    fileprivate func switchChildrenType(_ type: String) {
        self.childrenType = type
        self.setupSwipe()
        self.setupSwitcher()

        self.childrenViewController.loadData(true)

        self.setupNavigationActions()
        self.changeTheme()
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        self.setupNavigationActions()
    }

    fileprivate func setupNavigationActions() {
        let nav = (self.navigationController as! NavigationController)
        nav.resetState()
        nav.setBarActions([
            NavigationControllerBarAction(icon: .search, handler: { [weak self] () -> Void in
                self?.handleShowSearch()
            })
        ])

        // quick button only visible in current
        if (self.childrenType == "current") {
            nav.setQuickButtonActions([
                QuickButtonAction(icon: .iCursor, handler: { [weak self] () -> Void in
                    self?.handleCollectionRename()
                }),
                QuickButtonAction(icon: .plus, handler: { [weak self] () -> Void in
                    guard let this = self else {
                        return
                    }

//   TODO:                 (this.navigationController as? NavigationController)?.showCreateObject(this.entityId) { [weak self] (entity) -> Void in
//                        guard let entity = entity else {
//                            return
//                        }
//                        (self?.navigationController as? NavigationController)?.pushObject(.entityOld(entity: entity))
//                    }
                }),
                QuickButtonAction(icon: .folderOpen, handler: { [weak self] () -> Void in
                    self?.handleAddToCollection()
                })
            ])
        }
    }

    fileprivate func setupSwitcher() {
        self.childrenViewController.setSwitcherActions([
            SwitcherButtonAction(icon: .folder, active: self.childrenType == "current", callback: { [weak self] in
                self?.switchChildrenType("current")
            }),
            SwitcherButtonAction(icon: .check, active: self.childrenType == "old", callback: { [weak self] in
                self?.switchChildrenType("old")
            }),
        ])
    }

    fileprivate func setupSwipe() {
        if (self.childrenType == "current") {
            self.childrenViewController.setSwipeActions([
                ChildrenViewSwipeAction(action: .check, color: Stylesheet.collectionSwipeDoneBg, state: .state1, mode: .exit, handler: { [weak self] (entity) -> Void in
                    self?.handleDone(entity)
                }),
                ChildrenViewSwipeAction(action: .clock, color: Stylesheet.collectionSwipeLaterBg, state: .state3, mode: .switch, handler: { [weak self] (entity) -> Void in
                    self?.handleMoveLater(entity)
                }),
                ChildrenViewSwipeAction(action: .folderOpen, color: Stylesheet.collectionSwipeAddCollectionBg, state: .state4, mode: .switch, handler: { [weak self] (entity) -> Void in
                    self?.handleAddToCollection(entity)
                })
            ])
        } else {
            self.childrenViewController.setSwipeActions([
                ChildrenViewSwipeAction(action: .folder, color: Stylesheet.collectionSwipeDoneBg, state: .state3, mode: .exit, handler: { [weak self] (entity) -> Void in
                    self?.handleMoveCurrent(entity)
                }),
                ChildrenViewSwipeAction(action: .folderOpen, color: Stylesheet.collectionSwipeAddCollectionBg, state: .state4, mode: .switch, handler: { [weak self] (entity) -> Void in
                    self?.handleAddToCollection(entity)
                })
            ])
        }
    }

    fileprivate func handleCollectionRename() {
//        let alert = UIAlertController(title: "Name", message: "Enter a new name", preferredStyle: UIAlertController.Style.alert)
//        alert.addTextField(configurationHandler: { [weak self] (textField: UITextField!) in
//            textField.text = self?.trait.displayName
//            textField.isSecureTextEntry = false
//        })
//        alert.addAction(UIAlertAction(title: "Ok", style: .default, handler: { [weak self] (alertAction) -> Void in
//            guard let this = self else {
//                return
//            }
//            let newName = alert.textFields![0] as UITextField
//
//            if let collection = this.trait.trait as? CollectionFull, let name = newName.text {
//                collection.name = name
//                ExomindDSL.on(this.trait.entity).mutate.update(collection).execute()
//            }
//        }))
//        alert.addAction(UIAlertAction(title: "Cancel", style: .cancel, handler: nil))
//        self.present(alert, animated: true, completion: nil)
    }

    fileprivate func handleAddToCollection() {
//        guard let entityTrait = self.trait else {
//            return
//        }
//        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: entityTrait.entity)
    }

    fileprivate func handleShowSearch() {
//        (self.navigationController as? NavigationController)?.showSearch(self.entityId)
    }

    fileprivate func changeTheme() {
        if (self.childrenType == "old") {
            self.childrenViewController.setTheme(Stylesheet.collectionThemeDoneBg)
        } else {
            self.childrenViewController.setTheme(nil)
        }
    }

    fileprivate func handleItemClick(_ entity: EntityExt) {
//   TODO:     (self.navigationController as? NavigationController)?.pushObject(.entityOld(entity: entity))
    }

    fileprivate func handleDone(_ entity: EntityExt) {
//   TODO:     ExomindDSL.on(entity).relations.removeParent(parentId: self.entityId)
    }

    fileprivate func handleMoveCurrent(_ entity: EntityExt) {
//  TODO:      ExomindDSL.on(entity).relations.addParent(parentId: self.entityId)
    }

    fileprivate func handleCopyInbox(_ entity: EntityExt) {
//   TODO:     ExomindDSL.on(entity).relations.addParent(parentId: "inbox")
    }

    fileprivate func handleMoveLater(_ entity: EntityExt) {
//   TODO:     (self.navigationController as? NavigationController)?.showTimeSelector(forEntity: entity)
    }

    fileprivate func handleAddToCollection(_ entity: EntityExt) {
// TODO:        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: entity)
    }

    deinit {
        print("CollectionViewController > Deinit")
    }
}
