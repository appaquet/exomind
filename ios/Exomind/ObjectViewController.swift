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
    
    private var entityId: HCEntityId?
    private var entity: HCEntity?
    private var entityTrait: EntityTrait?
    private var specificTraitId: HCTraitId?
    
    private var rendered: Bool = false
    
    private var objectViewController: UIViewController?
    
    func populate(entity: HCEntity) {
        self.entityId = entity.id
        self.entity = entity
    }
    
    func populate(entityTrait: EntityTrait) {
        self.entity = entityTrait.entity
        self.entityId = entityTrait.entity.id
        self.specificTraitId = entityTrait.trait.traitId
    }
    
    func populate(entityId: HCEntityId) {
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
        
        if  let entityId = self.entityId {
            let entityQuery = self.querySet.executeQuery(HCQueries.Entities().withEntityId(entityId))
            
            if let newEntity = entityQuery.resultAsEntity() {
                self.entity = newEntity
            }
            
            if let entity = self.entity {
                if  let specificTraitId = self.specificTraitId,
                    let trait = entity.traitsById[specificTraitId] {
                    
                    self.entityTrait = EntityTrait(entity: entity, trait: trait)
                    self.title = entityTrait?.displayName
                    self.renderView()
                    
                } else if let entityTrait = EntityTrait(entity: entity) {
                    self.title = entityTrait.displayName
                    self.entityTrait = entityTrait
                    self.renderView()
                }
            }
            
            if let objectViewController = self.objectViewController,
                let entityTraitView = objectViewController as? EntityTraitView,
                let entityTrait = self.entityTrait {
                
                entityTraitView.loadEntityTrait(entityTrait)
            }
            
        }
    }
    
    func renderView() {
        if !self.rendered, let entityTrait = self.entityTrait {
            self.renderView(entityTrait)
            self.rendered = true
        }
    }
    
    func renderView(_ entityTrait: EntityTrait) {
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
        case .note(note: _):
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "NoteViewController") as! NoteViewController
            vc.loadEntityTrait(entityTrait)
            self.showVC(vc)
        case .link(link: _):
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "LinkViewController") as! LinkViewController
            vc.loadEntityTrait(entityTrait)
            self.showVC(vc)
        case .task(task: _):
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "TaskViewController") as! TaskViewController
            vc.loadEntityTrait(entityTrait)
            self.showVC(vc)
        case .collection(collection: _):
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "CollectionViewController") as! CollectionViewController
            vc.loadEntityTrait(entityTrait)
            self.showVC(vc)
        case .mind(mind: _):
            let vc = self.objectsStoryboard.instantiateViewController(withIdentifier: "CollectionViewController") as! CollectionViewController
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
    func loadEntityTrait(_ entityTrait: EntityTrait)
}
