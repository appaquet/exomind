import UIKit
import SnapKit
import Exocore

class EntityCreationViewController: ModalGridViewController {
    var parentId: EntityId?
    var callback: ((EntityExt?) -> Void)?

    convenience init(parentId: EntityId?, callback: ((EntityExt?) -> Void)?) {
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
                EntityCreationViewController.createTask(self?.parentId, callback: self?.callback)
                self?.close()
            }),
            GridIconsViewItem(label: "Note", icon: .pen, callback: { [weak self] (item) -> () in
                EntityCreationViewController.createNote(self?.parentId, callback: self?.callback)
                self?.close()
            }),
            GridIconsViewItem(label: "Email", icon: .envelopeOpen, callback: { [weak self] (item) -> () in
                EntityCreationViewController.createEmail(self?.parentId, callback: self?.callback)
                self?.close()
            }),
            GridIconsViewItem(label: "Collection", icon: .folderOpen, callback: { [weak self] (item) -> () in
                EntityCreationViewController.createCollection(self?.parentId, callback: self?.callback)
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

    static func createTask(_ parentId: EntityId?, callback: ((EntityExt?) -> Void)?) {
        do {
            var task = Exomind_Base_V1_Task()
            task.title = "New task"

            var builder = try MutationBuilder
                    .createEntity()
                    .returnEntities()
                    .putTrait(message: task)

            try Commands.addChildMutation(parentId: parentId ?? "inbox", builder: &builder)
            Commands.executeNewEntityMutation(mutation: builder.build(), callback: callback)
        } catch {
            print("Error creating task: \(error)")
        }
    }

    static func createNote(_ parentId: EntityId?, callback: ((EntityExt?) -> Void)?) {
        do {
            var note = Exomind_Base_V1_Note()
            note.title = "New note"

            var builder = try MutationBuilder
                    .createEntity()
                    .returnEntities()
                    .putTrait(message: note)

            try Commands.addChildMutation(parentId: parentId ?? "inbox", builder: &builder)
            Commands.executeNewEntityMutation(mutation: builder.build(), callback: callback)
        } catch {
            print("Error creating note: \(error)")
        }
    }

    static func createEmail(_ parentId: EntityId?, callback: ((EntityExt?) -> Void)?) {
        do {
            let email = Exomind_Base_V1_DraftEmail()

            var builder = try MutationBuilder
                    .createEntity()
                    .returnEntities()
                    .putTrait(message: email)

            try Commands.addChildMutation(parentId: parentId ?? "inbox", builder: &builder)
            Commands.executeNewEntityMutation(mutation: builder.build(), callback: callback)
        } catch {
            print("Error creating collection: \(error)")
        }
    }

    static func createCollection(_ parentId: EntityId?, callback: ((EntityExt?) -> Void)?) {
        do {
            var collection = Exomind_Base_V1_Collection()
            collection.name = "New collection"

            var builder = try MutationBuilder
                    .createEntity()
                    .returnEntities()
                    .putTrait(message: collection)

            try Commands.addChildMutation(parentId: parentId ?? "inbox", builder: &builder)
            Commands.executeNewEntityMutation(mutation: builder.build(), callback: callback)
        } catch {
            print("Error creating collection: \(error)")
        }
    }
}
