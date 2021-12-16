import UIKit
import SnapKit
import Exocore

class NoteViewController: VerticalLinearViewController, EntityTraitView {
    private var entity: EntityExt?
    private var noteTrait: TraitInstance<Exomind_Base_V1_Note>?
    private var modifiedNote: Exomind_Base_V1_Note?

    private var headerView: LabelledFieldView!
    private var titleField: MultilineTextField!
    private var richTextEditor: RichTextEditor!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance, fullEntity: Bool) {
        if !fullEntity {
            // we only load if it's full entity to prevent saving empty
            return
        }

        self.entity = entity
        self.noteTrait = entity.trait(withId: trait.id)

        if self.isViewLoaded && self.modifiedNote == nil {
            self.loadData()
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        self.createHeaderView()
        self.createRichTextEditor()
        self.loadData()
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        let nav = (self.navigationController as! NavigationController)
        nav.resetState()

        self.tabBarController?.tabBar.isHidden = true
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)

        self.saveNote()
        self.tabBarController?.tabBar.isHidden = false
    }

    private func createHeaderView() {
        self.titleField = MultilineTextField()
        titleField.onChanged = { [weak self] text -> Void in
            guard let this = self else {
                return
            }

            if this.modifiedNote == nil {
                this.modifiedNote = this.noteTrait?.message
            }
            this.modifiedNote?.title = this.titleField.text
            this.saveNote()
        }
        self.headerView = LabelledFieldView(label: "Title", fieldView: titleField)
        self.addLinearView(self.headerView)
    }

    private func createRichTextEditor() {
        self.richTextEditor = RichTextEditor(callback: { [weak self] (json) -> Void in
            guard let this = self else {
                return
            }

            if let url = json?["link"].string {
                if url.starts(with: "entity://") {
                    let entityId = url.replacingOccurrences(of: "entity://", with: "")
                    let obj = NavigationObject.entityId(id: EntityId(entityId))
                    (self?.navigationController as? NavigationController)?.pushObject(obj)
                } else if url.starts(with: "http") {
                    if let parsedUrl = URL(string: url) {
                        let sfVc = SFSafariHelper.getViewControllerForURL(parsedUrl)
                        this.present(sfVc, animated: true, completion: nil)
                    }
                }
            } else if let body = json?["content"].string {
                if this.noteTrait == nil {
                    // note is still loading, we don't accept any changes yet
                    return
                }

                if this.modifiedNote == nil {
                    this.modifiedNote = this.noteTrait?.message
                }
                this.modifiedNote?.body = body
                this.saveNote() // we don't care about saving every time since it's already debounced in javascript
            }
        })

        self.addChild(self.richTextEditor)
        self.richTextEditor.didMove(toParent: self)
        self.richTextEditor.viewDidLoad()
        self.richTextEditor.delegateScrollTo(self.scrollView)
        self.addLinearView(self.richTextEditor.view)
    }

    private func loadData() {
        if let note = self.modifiedNote ?? self.noteTrait?.message {
            self.titleField.text = note.title
            self.richTextEditor.setContent(note.body)
        }
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

                print("NoteViewController > Saving note")
                ExocoreClient.store.mutate(mutation: mutation)
            } catch {
                print("NoteViewController > Error mutating note: \(error)")
            }
        }
    }

    deinit {
        print("NoteViewController > Deinit")
    }
}
