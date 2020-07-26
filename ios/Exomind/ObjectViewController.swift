//
//  ObjectViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-11-16.
//  Copyright © 2015 Exomind. All rights reserved.
//

import UIKit

class ObjectViewController: UIViewController {
    fileprivate let objectsStoryboard: UIStoryboard = UIStoryboard(name: "Objects", bundle: nil)

    private var querySet: QuerySet!

    private var entityId: EntityId?
    private var entityOld: HCEntity?
    private var entity: EntityExt?
    private var entityTraitOld: EntityTraitOld?
    private var entityTrait: AnyTraitInstance?
    private var specificTraitId: TraitId?

    private var rendered: Bool = false

    private var objectViewController: UIViewController?

    func populate(entity: HCEntity) {
        self.entityId = entity.id
        self.entityOld = entity
    }

    func populate(entityTrait: EntityTraitOld) {
        self.entityOld = entityTrait.entity
        self.entityId = entityTrait.entity.id
        self.specificTraitId = entityTrait.trait.traitId
    }

    func populate(entity: EntityExt) {
        self.entityId = entity.id
        self.entity = entity
        self.entityTrait = entity.priorityTrait
    }

    func populate(entityTrait: AnyTraitInstance) {
        self.entity = entityTrait.entity
        self.entityId = entityTrait.entity?.id
        self.specificTraitId = entityTrait.trait.id
    }

    func populate(entityId: EntityId) {
        self.entityId = entityId
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.loadData()
    }

    func loadData() {
        if (self.querySet == nil) {
            self.querySet = DomainStore.instance.getQuerySet()
            self.querySet.onChange { [weak self] () -> Void in
                self?.loadData()
            }
        }

        if let trait = self.entityTrait {
            self.title = trait.displayName
            self.renderView()

        } else if let entityId = self.entityId {
            let entityQuery = self.querySet.executeQuery(HCQueries.Entities().withEntityId(entityId))

            if let newEntity = entityQuery.resultAsEntity() {
                self.entityOld = newEntity
            }

            if let entity = self.entityOld {
                if let specificTraitId = self.specificTraitId,
                   let trait = entity.traitsById[specificTraitId] {

                    self.entityTraitOld = EntityTraitOld(entity: entity, trait: trait)
                    self.title = entityTraitOld?.displayName
                    self.renderView()

                } else if let entityTrait = EntityTraitOld(entity: entity) {
                    self.title = entityTrait.displayName
                    self.entityTraitOld = entityTrait
                    self.renderView()
                }
            }

            if let objectViewController = self.objectViewController,
               let entityTraitView = objectViewController as? EntityTraitViewOld,
               let entityTrait = self.entityTraitOld {

                entityTraitView.loadEntityTrait(entityTrait)
            }
        }
    }

    func renderView() {
        if self.rendered {
            return
        }
        self.rendered = true

        if let entity = self.entity, let trait = self.entityTrait {
            self.renderView(entity: entity, trait: trait)

        } else if let entityTrait = self.entityTraitOld {
            self.renderViewOld(entityTrait)
        }
    }

    func renderView(entity: EntityExt, trait: AnyTraitInstance) {
        guard let traitType = trait.type else {
            return
        }

        switch traitType {
        case .collection:
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "CollectionViewController") as! CollectionViewController
            vc.loadEntityTrait(entity: entity, trait: trait)
            self.showVC(vc)

        case .note:
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "NoteViewController") as! NoteViewController
            vc.loadEntityTrait(entity: entity, trait: trait)
            self.showVC(vc)

        case .link:
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "LinkViewController") as! LinkViewController
            vc.loadEntityTrait(entity: entity, trait: trait)
            self.showVC(vc)

        default: break
        }
    }

    func renderViewOld(_ entityTrait: EntityTraitOld) {
        switch (entityTrait.traitType) {
        case .draftEmail(draftEmail: _):
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "DraftEmailViewController") as! DraftEmailViewController
            vc.loadEntityTrait(entityTrait)
            self.showVC(vc)
        case .email(email: _):
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "EmailViewController") as! EmailViewController
            vc.loadEntityTrait(entityTrait)
            self.showVC(vc)
        case .emailThread(emailThread: _):
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "EmailThreadViewController") as! EmailThreadViewController
            vc.loadEntityTrait(entityTrait)
            self.showVC(vc)
        case .task(task: _):
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "TaskViewController") as! TaskViewController
            vc.loadEntityTrait(entityTrait)
            self.showVC(vc)
        default:
            break
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        // reset states in navigation controller (buttons, etc.)
        (self.navigationController as? NavigationController)?.resetState()
    }

    func showVC(_ vc: UIViewController) {
        self.objectViewController = vc
        self.addChild(vc)
        vc.view.frame = CGRect(x: 0, y: 0, width: self.view.frame.size.width, height: self.view.frame.size.height);
        self.view.addSubview(vc.view)
        vc.didMove(toParent: self)
        vc.viewWillAppear(false)
        vc.viewDidAppear(false)
    }

    deinit {
        print("ObjectViewController > Deinit")
    }
}

protocol EntityTraitView {
    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance)
}

protocol EntityTraitViewOld {
    func loadEntityTrait(_ entityTrait: EntityTraitOld)
}
