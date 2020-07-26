//
//  DraftEmailViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-03-16.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit
import SnapKit
import CLTokenInputView

class DraftEmailViewController: VerticalLinearViewController, EntityTraitViewOld, CLTokenInputViewDelegate {
    fileprivate var entityTrait: EntityTraitOld!
    fileprivate var serverTrait: DraftEmailFull!
    fileprivate var localTrait: DraftEmailFull!

    fileprivate var fromField: UILabel!
    fileprivate var toField: ContactsField!
    fileprivate var ccField: ContactsField!
    fileprivate var subjectField: MultilineTextField!
    fileprivate var richTextEditor: RichTextEditor!

    fileprivate var integrations = [IntegrationSourceFull]()

    func loadEntityTrait(_ entityTrait: EntityTraitOld) {
        self.entityTrait = entityTrait
        self.render()
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.createViews()
    }

    func createViews() {
        self.fromField = UILabel()
        self.fromField.font = UIFont.systemFont(ofSize: 14)
        self.fromField.isUserInteractionEnabled = true
        self.fromField.addGestureRecognizer(UITapGestureRecognizer(target: self, action: #selector(handleFromTouch)))
        self.addLinearView(LabelledFieldView(label: "From", fieldView: self.fromField, betweenPadding: 15))

        self.toField = ContactsField(label: "To") { [weak self] in
            if let contacts = self?.toField.getContacts() {
                self?.localTrait.to = contacts
            }
        }
        self.addLinearView(self.toField.headerView)

        self.ccField = ContactsField(label: "CC") { [weak self] in
            if let contacts = self?.ccField.getContacts() {
                self?.localTrait.cc = contacts
            }
        }
        self.addLinearView(self.ccField.headerView)

        self.subjectField = MultilineTextField()
        self.subjectField.onChanged = { [weak self] (text) in
            self?.localTrait.subject = text
        }
        self.addLinearView(LabelledFieldView(label: "Subject", fieldView: self.subjectField))

        self.createRichTextView()
    }

    func createRichTextView() {
        self.richTextEditor = RichTextEditor(callback: { [weak self] (json) -> Void in
            if let body = json?["content"].string {
                self?.localTrait.parts = [EmailPartHtmlFull(body: body)]

                // we don't care to save everytime since it's already debounced in javascript
                self?.saveEmail()
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
        let nav = (self.navigationController as! NavigationController)
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

    func render() {
        if !self.isViewLoaded || self.localTrait != nil {
            return
        }
        
        guard   let serverTrait = entityTrait.trait as? DraftEmailFull,
                let localTrait = serverTrait.clone() as? DraftEmailFull
            else { return }
        
        self.serverTrait = serverTrait
        self.localTrait = localTrait
        self.integrations = SessionStore.integrations()
            .compactMap { entityTrait in
                switch (entityTrait.traitType) {
                case let .integration(integration: int) where int.typ == "google":
                    return IntegrationSourceFull(data: [:], integrationKey: int.key, integrationName: "google")
                default:
                    return nil
                }
            }

        if let htmlPart = (self.localTrait.parts.compactMap { $0 as? EmailPartHtmlFull }).first {
            self.richTextEditor.setContent(htmlPart.body)
        }
        
        if self.localTrait.from == nil {
            self.localTrait.from = self.integrations.first
        }

        self.subjectField.text = self.localTrait.subject ?? ""
        self.fromField.text = self.localTrait.from?.integrationKey
        self.toField.setContacts(self.localTrait.to.compactMap { $0 as? ContactFull })
        self.ccField.setContacts(self.localTrait.cc.compactMap { $0 as? ContactFull })
    }

    func saveEmail() {
        if !self.localTrait.equals(self.serverTrait) {
            ExomindDSL.on(self.entityTrait.entity).mutate.put(self.localTrait).execute()
        }
    }

    func handleSendClick() {
        self.localTrait.sendingDate = Date()
        self.saveEmail()
        let _ = self.navigationController?.popViewController(animated: true)
    }

    func handleDelete() {
        ExomindDSL.on(self.entityTrait.entity).mutate.remove(self.localTrait).execute()
        let _ = self.navigationController?.popViewController(animated: true)
    }

    @objc func handleFromTouch() {
        let vc = UIAlertController(title: nil, message: nil, preferredStyle: .actionSheet)
        for integration in self.integrations {
            vc.addAction(UIAlertAction(title: integration.integrationKey, style: .default, handler: { [weak self] (action) -> Void in
                self?.localTrait.from = integration
                self?.render()
            }))
        }
        vc.addAction(UIAlertAction(title: "Cancel", style: .cancel, handler: nil))
        self.present(vc, animated: true, completion: nil)
    }

    override func viewWillDisappear(_ animated: Bool) {
        self.saveEmail()
    }

    deinit {
        print("NewDraftEmailViewController > Deinit")
    }
}

private class ContactWrapper: NSObject {
    let contact: ContactFull
    init(contact: ContactFull) {
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

    func createField() {
        UITextField.appearance(whenContainedInInstancesOf: [CLTokenInputView.self]).font = UIFont.systemFont(ofSize: 14)
        self.tokensField = CLTokenInputView()
        self.tokensField.delegate = self
        self.tokensField.backgroundColor = UIColor.white
        self.headerView = LabelledFieldView(label: self.label, fieldView: self.tokensField, betweenPadding: 0)
    }

    @objc func tokenInputView(_ view: CLTokenInputView, didChangeText text: String?) {
        if let text = self.tokensField.text {
            if (text.hasSuffix(" ") || text.hasSuffix(",")) {
                self.maybeAddCurrentText()
            }
        }
    }

    @objc fileprivate func tokenInputView(_ view: CLTokenInputView, didRemove token: CLToken) {
        if !preventOnChange {
            self.onChange?()
        }
    }

    @objc fileprivate func tokenInputViewDidEndEditing(_ view: CLTokenInputView) {
        self.maybeAddCurrentText()
    }

    func maybeAddCurrentText() {
        if let text = self.tokensField.text, !preventOnChange {
            // TODO: better validation...
            if (text != "" && text.contains("@")) {
                let trimmed = text.trimmingCharacters(in: CharacterSet(charactersIn: ", \t\n"))
                let contact = ContactFull(email: trimmed, name: nil)
                self.tokensField.add(self.contactToToken(contact))
                self.onChange?()
            }
        }
    }

    func setContacts(_ contacts: [ContactFull]) {
        self.preventOnChange = true
        self.tokensField.allTokens.forEach { (token) -> () in
            self.tokensField.remove(token)
        }
        contacts.forEach { contact in
            self.tokensField.add(self.contactToToken(contact))
        }
        self.preventOnChange = false
    }

    func getContacts() -> [ContactFull] {
        return self.tokensField.allTokens.compactMap { token in
            return (token.context as? ContactWrapper)?.contact
        }
    }

    fileprivate func contactToToken(_ contact: ContactFull) -> CLToken {
        let displayName = EmailsLogic.formatContact(contact)
        return CLToken(displayText: displayName, context: ContactWrapper(contact: contact) as NSObject)
    }

    deinit {
        print("ContactsField > Deinit")
    }
}
