import UIKit
import SnapKit
import Exocore

class NoteViewController: UIViewController, EntityTraitView {
    private var entity: EntityExt!
    private var noteTrait: TraitInstance<Exomind_Base_Note>?
    private var modifiedNote: Exomind_Base_Note?

    private var headerView: LabelledFieldView!
    private var titleField: MultilineTextField!
    private var richTextEditor: RichTextEditor!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance) {
        self.entity = entity
        self.noteTrait = entity.trait(withId: trait.id)
        self.modifiedNote = entity.trait(withId: trait.id)?.message
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        self.createHeaderView()
        self.createWebView()
        self.render()

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

    fileprivate func createHeaderView() {
        self.titleField = MultilineTextField()
        titleField.onChanged = { [weak self] text -> Void in
            self?.handleTitleChange()
        }
        self.headerView = LabelledFieldView(label: "Title", fieldView: titleField)
        self.view.addSubview(self.headerView)
    }

    fileprivate func createWebView() {
        self.richTextEditor = RichTextEditor(callback: { [weak self] (json) -> Void in
            if let body = json?["content"].string {
                self?.modifiedNote?.body = body
                self?.saveNote() // we don't care to save every time since it's already debounced in javascript
            }
        })
        self.addChild(self.richTextEditor)
        self.view.addSubview(self.richTextEditor.view)
        self.richTextEditor.didMove(toParent: self)
        self.richTextEditor.viewDidLoad()
    }

    fileprivate func render() {
        guard let localNote = self.modifiedNote else {
            return
        }

        self.titleField.text = localNote.title
        self.richTextEditor.setContent(localNote.body)
    }

    fileprivate func handleTitleChange() {
        self.modifiedNote?.title = self.titleField.text
        self.saveNote()
    }

    fileprivate func saveNote() {
        guard   let initialNote = self.noteTrait,
                let modifiedNote = self.modifiedNote
                else {
            return
        }

        if !initialNote.message.isEqualTo(message: modifiedNote) {
            do {
                let mutation = try MutationBuilder
                        .updateEntity(entityId: self.entity.id)
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

