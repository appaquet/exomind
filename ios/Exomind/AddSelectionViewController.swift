//
//  AddSelectorViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-12-08.
//  Copyright © 2015 Exomind. All rights reserved.
//

import UIKit
import SnapKit

class AddSelectionViewController: ModalGridViewController {
    var parentId: String?
    var callback: ((HCEntity?) -> Void)?

    convenience init(parentId: String?, callback: ((HCEntity?) -> Void)?) {
        self.init(nibName: nil, bundle: nil)
        self.parentId = parentId
        self.callback = callback
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.createView()
    }

    func createView() {
        let items: [GridIconsViewItem] = [
                GridIconsViewItem(label: "Task", icon: .check, callback: { [weak self] (item) -> () in
                    AddSelectionViewController.createTask(self?.parentId, callback: self?.callback)
                    self?.close()
                }),
                GridIconsViewItem(label: "Note", icon: .pen, callback: { [weak self] (item) -> () in
                    AddSelectionViewController.createNote(self?.parentId, callback: self?.callback)
                    self?.close()
                }),
                GridIconsViewItem(label: "Email", icon: .envelopeOpen, callback: { [weak self] (item) -> () in
                    AddSelectionViewController.createEmail(self?.parentId, callback: self?.callback)
                    self?.close()
                }),
                GridIconsViewItem(label: "Collection", icon: .folderOpen, callback: { [weak self] (item) -> () in
                    AddSelectionViewController.createCollection(self?.parentId, callback: self?.callback)
                    self?.close()
                }),
        ]

        let view = GridIconsView(items: items)
        view.squarePerRow = 2
        view.initView()
        self.view.addSubview(view)
        view.snp.makeConstraints { (make) in
            make.size.equalTo(self.view.snp.size)
            make.center.equalTo(self.view.snp.center)
        }
    }

    static func createTask(_ parentId: HCEntityId?, callback: ((HCEntity?) -> Void)?) {
        let task = TaskBuilder()
        task.title = "New task"
        let child = EntityRelations.buildChild(parentId: parentId ?? "inbox").toBuilder() as! HCTraitBuilder
        ExomindDSL.newEntity(traitsBuilder: [task, child]).onProcessed { (cmd, entity) -> Void in
            callback?(entity)
        }
    }

    static func createNote(_ parentId: HCEntityId?, callback: ((HCEntity?) -> Void)?) {
        let note = NoteBuilder()
        note.title = "New note"
        let child = EntityRelations.buildChild(parentId: parentId ?? "inbox").toBuilder() as! HCTraitBuilder
        ExomindDSL.newEntity(traitsBuilder: [note, child]).onProcessed { (cmd, entity) -> Void in
            callback?(entity)
        }
    }

    static func createEmail(_ parentId: HCEntityId?, callback: ((HCEntity?) -> Void)?) {
        let draftEmail = DraftEmailBuilder()
        draftEmail.attachments = []
        draftEmail.to = []
        draftEmail.cc = []
        draftEmail.bcc = []
        draftEmail.parts = []
        let child = EntityRelations.buildChild(parentId: parentId ?? "inbox").toBuilder() as! HCTraitBuilder
        ExomindDSL.newEntity(traitsBuilder: [draftEmail, child]).onProcessed { (cmd, entity) -> Void in
            callback?(entity)
        }
    }

    static func createCollection(_ parentId: HCEntityId?, callback: ((HCEntity?) -> Void)?) {
        let collection = CollectionBuilder()
        collection.name = "New collection"
        let child = EntityRelations.buildChild(parentId: parentId ?? "inbox").toBuilder() as! HCTraitBuilder
        ExomindDSL.newEntity(traitsBuilder: [collection, child]).onProcessed { (cmd, entity) -> Void in
            callback?(entity)
        }
    }
}
