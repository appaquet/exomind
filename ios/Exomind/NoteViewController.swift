import UIKit
import SnapKit
import Exocore

class NoteViewController: UIViewController, EntityTraitView {
    private var partialEntity: EntityExt!
    private var noteTraitId: String!

    private var entityQuery: QueryHandle?
    private var entity: EntityExt?
    private var noteTrait: TraitInstance<Exomind_Base_Note>?
    private var modifiedNote: Exomind_Base_Note?

    private var headerView: LabelledFieldView!
    private var titleField: MultilineTextField!
    private var richTextEditor: RichTextEditor!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance) {
        self.partialEntity = entity
        self.noteTraitId = trait.id
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        self.createHeaderView()
        self.createRichTextEditor()
        self.loadEntity()

        self.headerView.snp.makeConstraints { (make) in
            make.width.equalTo(self.view.snp.width)
            make.centerX.equalTo(self.view.snp.centerX)
            make.top.equalTo(self.view.snp.topMargin)
        }

        self.richTextEditor.view.snp.makeConstraints { (make) in
            make.top.equalTo(self.headerView.snp.bottom)
            make.bottom.equalTo(self.view.snp.bottom)
            make.width.equalTo(self.view.snp.width)
            make.centerX.equalTo(self.view.snp.centerX)
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        let nav = (self.navigationController as! NavigationController)
        nav.resetState()
    }

    private func createHeaderView() {
        self.titleField = MultilineTextField()
        titleField.onChanged = { [weak self] text -> Void in
            self?.handleTitleChange()
        }
        self.headerView = LabelledFieldView(label: "Title", fieldView: titleField)
        self.view.addSubview(self.headerView)
    }

    private func createRichTextEditor() {
        self.richTextEditor = RichTextEditor(callback: { [weak self] (json) -> Void in
            if let body = json?["content"].string {
                self?.modifiedNote?.body = body
                self?.saveNote() // we don't care about saving every time since it's already debounced in javascript
            }
        })
        self.addChild(self.richTextEditor)
        self.view.addSubview(self.richTextEditor.view)
        self.richTextEditor.didMove(toParent: self)
        self.richTextEditor.viewDidLoad()
    }

    private func loadEntity() {
        let query = QueryBuilder.withId(self.partialEntity.id).build()
        self.entityQuery = ExocoreClient.store.query(query: query, onChange: { [weak self] (status, res) in
            guard let this = self,
                  let res = res,
                  let entity = res.entities.first?.entity.toExtension() else {
                return
            }

            DispatchQueue.main.async {
                this.entity = entity
                this.noteTrait = entity.trait(withId: this.noteTraitId)
                this.modifiedNote = entity.trait(withId: this.noteTraitId)?.message
                if let note = this.modifiedNote {
                    this.titleField.text = note.title
                    this.richTextEditor.setContent(note.body)
                }
            }
        })
    }

    private func handleTitleChange() {
        self.modifiedNote?.title = self.titleField.text
        self.saveNote()
    }

    private func saveNote() {
        guard   let entity = self.entity,
                let initialNote = self.noteTrait,
                let modifiedNote = self.modifiedNote
                else {
            return
        }

        if !initialNote.message.isEqualTo(message: modifiedNote) {
            do {
                let mutation = try MutationBuilder
                        .updateEntity(entityId: entity.id)
                        .putTrait(message: modifiedNote, traitId: initialNote.id)
                        .build()

                ExocoreClient.store.mutate(mutation: mutation)
            } catch {
                print("NoteViewController > Error mutating note: \(error)")
            }
        }
    }

    override func viewWillDisappear(_ animated: Bool) {
        self.saveNote()
    }

    deinit {
        print("NoteViewController > Deinit")
    }
}

