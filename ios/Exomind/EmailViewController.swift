//
//  EmailViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-11-20.
//  Copyright © 2015 Exomind. All rights reserved.
//

import UIKit

class EmailViewController: VerticalLinearViewController, EntityTraitView {
    fileprivate let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    var entityTrait: EntityTrait!

    var webview: EmailBodyWebView!
    var fromLabel: UILabel!
    var toLabel: UILabel!
    var subjectLabel: UILabel!
    
    func loadEntityTrait(_ entityTrait: EntityTrait) {
        self.entityTrait = entityTrait
        self.render()
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.createFieldsView()
        self.createWebView()
        self.render()
    }

    func createWebView() {
        self.webview = EmailBodyWebView()
        self.webview.initialize()
        self.webview.onLinkClick = { url -> Bool in
            UIApplication.shared.open(url as URL, options: convertToUIApplicationOpenExternalURLOptionsKeyDictionary([:]), completionHandler: { (success) in
            })
            return true
        }
        self.addLinearView(self.webview)
        self.webview.loadHTML("Loading...")
    }

    func createFieldsView() {
        self.fromLabel = UILabel()
        self.fromLabel.font = UIFont.systemFont(ofSize: 14)
        self.fromLabel.numberOfLines = 0
        let fromField = LabelledFieldView(label: "From", fieldView: self.fromLabel, betweenPadding: 10)
        self.addLinearView(fromField)

        self.toLabel = UILabel()
        self.toLabel.font = UIFont.systemFont(ofSize: 14)
        self.toLabel.numberOfLines = 0
        let toField = LabelledFieldView(label: "To", fieldView: self.toLabel, betweenPadding: 10)
        self.addLinearView(toField)

        self.subjectLabel = UILabel()
        self.subjectLabel.font = UIFont.systemFont(ofSize: 14)
        self.subjectLabel.numberOfLines = 0
        let subjectField = LabelledFieldView(label: "Subject", fieldView: self.subjectLabel, betweenPadding: 10)
        self.addLinearView(subjectField)
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)

        let nav = (self.navigationController as! NavigationController)
        nav.resetState()
        nav.setQuickButtonActions([
                  QuickButtonAction(icon: .reply, handler: { [weak self]() -> Void in
                      self?.handleReply()
                  }),
                  QuickButtonAction(icon: .replyAll, handler: { [weak self]() -> Void in
                      self?.handleReplyAll()
                  }),
                  QuickButtonAction(icon: .forward, handler: { [weak self]() -> Void in
                      self?.handleForward()
                  }),
                  QuickButtonAction(icon: .folderOpen, handler: { [weak self]() -> Void in
                      self?.handleAddToCollection()
                  }),
          ])
    }

    func render() {
        if self.isViewLoaded, let email = self.entityTrait?.trait as? EmailFull {
            self.webview.loadEmailEntity(self.entityTrait, short: false)

            self.fromLabel.text = EmailsLogic.formatContact(email.from)
            self.subjectLabel.text = email.subject ?? "(no subject)"
            let joined = (email.to + email.cc).map { $0.name ?? $0.email }
            self.toLabel.text = joined.joined(separator: ", ")
        }
    }

    func handleReply() {
        EmailsLogic.createReplyEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
            guard let this = self, let entity = entity else { return }
            (this.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
        }
    }

    func handleReplyAll() {
        EmailsLogic.createReplyAllEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
            guard let this = self, let entity = entity else { return }
            (this.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
        }
    }

    func handleForward() {
        EmailsLogic.createForwardEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
            guard let this = self, let entity = entity else { return }
            (this.navigationController as? NavigationController)?.pushObject(.entity(entity: entity))
        }
    }

    func handleAddToCollection() {
        let vc = self.mainStoryboard.instantiateViewController(withIdentifier: "CollectionSelectorViewController") as! CollectionSelectorViewController
        vc.forEntity = self.entityTrait.entity
        self.present(vc, animated: true, completion: nil)
    }

    deinit {
        print("EmailViewController > Deinit")
    }
}

// Helper function inserted by Swift 4.2 migrator.
fileprivate func convertToUIApplicationOpenExternalURLOptionsKeyDictionary(_ input: [String: Any]) -> [UIApplication.OpenExternalURLOptionsKey: Any] {
	return Dictionary(uniqueKeysWithValues: input.map { key, value in (UIApplication.OpenExternalURLOptionsKey(rawValue: key), value)})
}
