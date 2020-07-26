import UIKit
import SnapKit
import Exocore

class NoteViewController: UIViewController, EntityTraitView {
    private var entity: EntityExt!
    private var serverNote: Exomind_Base_Note?
    private var localNote: Exomind_Base_Note?

    private var headerView: LabelledFieldView!
    private var titleField: MultilineTextField!
    private var richTextEditor: RichTextEditor!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance) {
        self.entity = entity
        self.serverNote = entity.trait(withId: trait.id)?.message

        let localNote: Exomind_Base_Note? = entity.trait(withId: trait.id)?.message
        self.localNote = localNote?.expensiveClone()
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
            if let body = json?["content"].string, var note = self?.localNote {
                note.body = body
                self?.saveNote() // we don't care to save every time since it's already debounced in javascript
            }
        })
        self.addChild(self.richTextEditor)
        self.view.addSubview(self.richTextEditor.view)
        self.richTextEditor.didMove(toParent: self)
        self.richTextEditor.viewDidLoad()
    }

    fileprivate func render() {
        guard let localNote = self.localNote else { return }

        self.titleField.text = localNote.title
        self.richTextEditor.setContent(localNote.body)
    }

    fileprivate func handleTitleChange() {
        self.localNote?.title = self.titleField.text
        self.saveNote()
    }

    fileprivate func saveNote() {
        guard   let serverNote = self.serverNote,
                let localNote = self.localNote
                else {
            return
        }

        if !serverNote.isEqualTo(message: localNote) {
            //TODO: ExomindDSL.on(entityTrait.entity).mutate.put(localNote).execute()
        }
    }

    override func viewWillDisappear(_ animated: Bool) {
        self.saveNote()
    }

    deinit {
        print("NoteViewController > Deinit")
    }
}

