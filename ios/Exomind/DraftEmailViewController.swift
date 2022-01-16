import UIKit
import SnapKit
import CLTokenInputView

import Exocore

class DraftEmailViewController: VerticalLinearViewController, EntityTraitView, CLTokenInputViewDelegate {
    private var entity: EntityExt?
    private var draftTrait: TraitInstance<Exomind_Base_V1_DraftEmail>?
    private var modifiedDraft: Exomind_Base_V1_DraftEmail?

    private var fromField: UILabel!
    private var toField: ContactsField!
    private var ccField: ContactsField!
    private var subjectField: MultilineTextField!
    private var richTextEditor: RichTextEditor!

    private var accounts = [Account]()
    private var accountQuery: QueryHandle?

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance, fullEntity: Bool) {
        if !fullEntity {
            // we only load if it's full entity to prevent saving empty
            return
        }

        self.entity = entity
        self.draftTrait = entity.trait(withId: trait.id)

        if self.accounts.isEmpty {
            let query = QueryBuilder.withTrait(Exomind_Base_V1_Account.self).build()
            self.accountQuery = ExocoreClient.store.query(query: query) { [weak self] (status, results) in
                guard let results = results else {
                    return
                }

                self?.accounts = results.entities.compactMap({ (result) in
                    let entity = result.entity.toExtension()
                    guard let account: TraitInstance<Exomind_Base_V1_Account> = entity.traitOfType(Exomind_Base_V1_Account.self) else {
                        return nil
                    }
                    return Account(entity: entity, trait: account)
                })

                DispatchQueue.main.async { [weak self] in
                    self?.loadData()
                }
            }
        }

        if self.isViewLoaded && self.modifiedDraft == nil {
            self.loadData()
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.createViews()
        self.loadData()
    }

    private func createViews() {
        self.fromField = UILabel()
        self.fromField.font = UIFont.systemFont(ofSize: 14)
        self.fromField.isUserInteractionEnabled = true
        self.fromField.addGestureRecognizer(UITapGestureRecognizer(target: self, action: #selector(handleFromTouch)))
        self.addLinearView(LabelledFieldView(label: "From", fieldView: self.fromField, betweenPadding: 15))

        self.toField = ContactsField(label: "To") { [weak self] in
            guard let this = self else {
                return
            }

            if this.modifiedDraft == nil {
                this.modifiedDraft = this.draftTrait?.message
            }

            let contacts = this.toField.getContacts()
            this.modifiedDraft?.to = contacts
            this.saveDraft()
        }
        self.addLinearView(self.toField.headerView)

        self.ccField = ContactsField(label: "CC") { [weak self] in
            guard let this = self else {
                return
            }

            if this.modifiedDraft == nil {
                this.modifiedDraft = this.draftTrait?.message
            }

            let contacts = this.ccField.getContacts()
            this.modifiedDraft?.cc = contacts
            this.saveDraft()
        }
        self.addLinearView(self.ccField.headerView)

        self.subjectField = MultilineTextField()
        self.subjectField.onChanged = { [weak self] (text) in
            guard let this = self else {
                return
            }

            if this.modifiedDraft == nil {
                this.modifiedDraft = this.draftTrait?.message
            }

            this.modifiedDraft?.subject = text
            this.saveDraft()
        }
        self.addLinearView(LabelledFieldView(label: "Subject", fieldView: self.subjectField))

        self.createRichTextView()
    }

    func createRichTextView() {
        self.richTextEditor = RichTextEditor(callback: { [weak self] (json) -> Void in
            guard let this = self else {
                return
            }

            if let body = json?["content"].string {
                if this.modifiedDraft == nil {
                    this.modifiedDraft = this.draftTrait?.message
                }

                var part = Exomind_Base_V1_EmailPart()
                part.body = body
                part.mimeType = "text/html"
                this.modifiedDraft?.parts = [part]

                // we don't care to save everytime since it's already debounced in javascript
                this.saveDraft()
            }
        })
        self.addChild(self.richTextEditor)
        self.richTextEditor.didMove(toParent: self)
        self.richTextEditor.viewDidLoad()
        self.richTextEditor.delegateScrollTo(self.scrollView)

        self.addLinearView(self.richTextEditor.view)
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)

        if let nav = self.navigationController as? NavigationController {
            nav.resetState()
            nav.setBarActions([
                NavigationControllerBarAction(icon: .paperPlane, handler: { [weak self] () -> Void in
                    self?.handleSendClick()
                })
            ])
            nav.setQuickButtonActions([
                QuickButtonAction(icon: .times, handler: { [weak self] () -> Void in
                    self?.handleDelete()
                })
            ])
        }
    }

    func loadData() {
        if !self.isViewLoaded {
            return
        }

        guard var draft = self.modifiedDraft ?? self.draftTrait?.message else {
            return
        }

        if !draft.hasAccount {
            if let firstAccount = self.accounts.first {
                draft.account = firstAccount.reference
            }
        }

        self.fromField.text = accounts.first(where: { $0.reference == draft.account })?.trait.message.name
        self.toField.setContacts(draft.to)
        self.ccField.setContacts(draft.cc)
        self.subjectField.text = draft.subject
        self.richTextEditor.setContent(draft.parts.first?.body ?? "")
    }

    private func saveDraft() {
        guard   let entity = self.entity,
                let initialDraft = self.draftTrait,
                let modifiedDraft = self.modifiedDraft
                else {
            return
        }

        if !initialDraft.message.isEqualTo(message: modifiedDraft) {
            do {
                let mutation = try MutationBuilder
                        .updateEntity(entityId: entity.id)
                        .putTrait(message: modifiedDraft, traitId: initialDraft.id)
                        .build()

                ExocoreClient.store.mutate(mutation: mutation)
            } catch {
                print("DraftEmailViewController > Error mutating note: \(error)")
            }
        }
    }

    private func handleSendClick() {
        // TODO:        let _ = self.navigationController?.popViewController(animated: true)
    }

    private func handleDelete() {
        guard   let entity = self.entity,
                let initialDraft = self.draftTrait
                else {
            return
        }
        let mutation = MutationBuilder
                .updateEntity(entityId: entity.id)
                .deleteTrait(traitId: initialDraft.id)
                .build()

        ExocoreClient.store.mutate(mutation: mutation)

        let _ = self.navigationController?.popViewController(animated: true)
    }

    @objc func handleFromTouch() {
        let vc = UIAlertController(title: nil, message: nil, preferredStyle: .actionSheet)
        for account in self.accounts {
            vc.addAction(UIAlertAction(title: account.trait.message.name, style: .default, handler: { [weak self] (action) -> Void in
                var accountRef = Exocore_Store_Reference()
                accountRef.entityID = account.entity.id
                accountRef.traitID = account.trait.id

                self?.modifiedDraft?.account = account.reference
                self?.loadData()
            }))
        }
        vc.addAction(UIAlertAction(title: "Cancel", style: .cancel, handler: nil))
        self.present(vc, animated: true, completion: nil)
    }

    override func viewWillDisappear(_ animated: Bool) {
        self.saveDraft()
    }

    deinit {
        print("NewDraftEmailViewController > Deinit")
    }
}

private class ContactWrapper: NSObject {
    let contact: Exomind_Base_V1_Contact

    init(contact: Exomind_Base_V1_Contact) {
        self.contact = contact
    }
}

private class ContactsField: NSObject, CLTokenInputViewDelegate {
    private var label: String!
    private var tokensField: CLTokenInputView!
    fileprivate var headerView: LabelledFieldView!

    var onChange: (() -> Void)?
    private var preventOnChange: Bool = false

    init(label: String, onChange: (() -> Void)?) {
        super.init()
        self.label = label
        self.onChange = onChange
        self.createField()
    }

    private func createField() {
        UITextField.appearance(whenContainedInInstancesOf: [CLTokenInputView.self]).font = UIFont.systemFont(ofSize: 14)
        self.tokensField = CLTokenInputView()
        self.tokensField.delegate = self
        self.tokensField.backgroundColor = UIColor.systemBackground
        self.headerView = LabelledFieldView(label: self.label, fieldView: self.tokensField, betweenPadding: 0)
    }

    func tokenInputView(_ view: CLTokenInputView, didChangeText text: String?) {
        if let text = self.tokensField.text {
            if (text.hasSuffix(" ") || text.hasSuffix(",")) {
                self.maybeAddCurrentText()
            }
        }
    }

    func tokenInputView(_ view: CLTokenInputView, didRemove token: CLToken) {
        if !preventOnChange {
            self.onChange?()
        }
    }

    func tokenInputViewDidEndEditing(_ view: CLTokenInputView) {
        self.maybeAddCurrentText()
    }

    private func maybeAddCurrentText() {
        if let text = self.tokensField.text, !preventOnChange {
            // TODO: better validation...
            if (text != "" && text.contains("@")) {
                var contact = Exomind_Base_V1_Contact()
                contact.email = text.trimmingCharacters(in: CharacterSet(charactersIn: ", \t\n"))

                self.tokensField.add(self.contactToToken(contact))
                self.onChange?()
            }
        }
    }

    fileprivate func setContacts(_ contacts: [Exomind_Base_V1_Contact]) {
        self.preventOnChange = true
        self.tokensField.allTokens.forEach { (token) -> () in
            self.tokensField.remove(token)
        }
        contacts.forEach { contact in
            self.tokensField.add(self.contactToToken(contact))
        }
        self.preventOnChange = false
    }

    fileprivate func getContacts() -> [Exomind_Base_V1_Contact] {
        self.tokensField.allTokens.compactMap { token in
            (token.context as? ContactWrapper)?.contact
        }
    }

    private func contactToToken(_ contact: Exomind_Base_V1_Contact) -> CLToken {
        let displayName = Emails.formatContact(contact)
        return CLToken(displayText: displayName, context: ContactWrapper(contact: contact) as NSObject)
    }

    deinit {
        print("ContactsField > Deinit")
    }
}

private struct Account {
    let entity: EntityExt
    let trait: TraitInstance<Exomind_Base_V1_Account>

    var reference: Exocore_Store_Reference {
        get {
            var ref = Exocore_Store_Reference()
            ref.entityID = entity.id
            ref.traitID = trait.id
            return ref
        }
    }
}

