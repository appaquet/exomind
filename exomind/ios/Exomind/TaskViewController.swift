import UIKit
import Exocore

class TaskViewController: UIViewController, EntityTraitView {
    @IBOutlet weak var taskNameField: UITextField!

    private var entity: EntityExt!
    private var taskTrait: TraitInstance<Exomind_Base_V1_Task>!
    private var changed: Bool = false

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance, fullEntity: Bool) {
        self.entity = entity
        self.taskTrait = entity.trait(withId: trait.id)
    }

    override func viewDidLoad() {
        self.taskNameField.text = self.taskTrait.displayName
    }

    override func viewDidAppear(_ animated: Bool) {
        self.taskNameField.becomeFirstResponder()
        self.taskNameField.selectAll(self)
    }

    @IBAction func nameChanged(_ sender: AnyObject) {
        self.changed = true
    }

    override func viewWillDisappear(_ animated: Bool) {
        if let text = self.taskNameField.text, self.changed && text != "" {
            var task = self.taskTrait.message
            task.title = text

            do {
                let mutation = try MutationBuilder
                        .updateEntity(entityId: self.entity.id)
                        .putTrait(message: task, traitId: self.taskTrait.id)
                        .build()

                ExocoreClient.store.mutate(mutation: mutation)
            } catch {
                print("TaskViewController > Error mutating \(error)")
            }
        }
    }
}
