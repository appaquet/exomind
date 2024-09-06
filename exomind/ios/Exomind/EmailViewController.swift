import UIKit
import Exocore

class EmailViewController: VerticalLinearViewController, EntityTraitView {
    private let mainStoryboard: UIStoryboard = UIStoryboard(name: "Main", bundle: nil)

    private var entity: EntityExt!
    private var email: TraitInstance<Exomind_Base_V1_Email>!

    private var webview: EmailBodyWebView!
    private var fromLabel: UILabel!
    private var toLabel: UILabel!
    private var subjectLabel: UILabel!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance, fullEntity: Bool) {
        self.entity = entity
        self.email = entity.trait(withId: trait.id)
        self.loadData()
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.createFieldsView()
        self.createWebView()
        self.loadData()
    }

    private func createWebView() {
        self.webview = EmailBodyWebView()
        self.webview.initialize()
        self.webview.onLinkClick = { url -> Bool in
            UIApplication.shared.open(url as URL, options: [:], completionHandler: { (success) in
            })
            return true
        }
        self.addLinearView(self.webview)
        self.webview.loadHTML("Loading...")
    }

    private func createFieldsView() {
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

        if let nav = self.navigationController as? NavigationController {
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
    }

    private func loadData() {
        guard let email = self.email, self.isViewLoaded else {
            return
        }

        self.webview.loadEmailEntity(email.message.parts, short: false)

        self.fromLabel.text = Emails.formatContact(email.message.from)
        self.subjectLabel.text = email.message.subject.nonEmpty() ?? "(no subject)"

        let joined = (email.message.to + email.message.cc).map {
            $0.name.nonEmpty() ?? $0.email
        }
        self.toLabel.text = joined.joined(separator: ", ")
    }

    func handleReply() {
        // TODO:
//        EmailsLogic.createReplyEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
//            guard let this = self, let entity = entity else { return }
//            (this.navigationController as? NavigationController)?.pushObject(.entityOld(entity: entity))
//        }
    }

    func handleReplyAll() {
        // TODO:
//        EmailsLogic.createReplyAllEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
//            guard let this = self, let entity = entity else { return }
//            (this.navigationController as? NavigationController)?.pushObject(.entityOld(entity: entity))
//        }
    }

    func handleForward() {
        // TODO:
//        EmailsLogic.createForwardEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
//            guard let this = self, let entity = entity else { return }
//            (this.navigationController as? NavigationController)?.pushObject(.entityOld(entity: entity))
//        }
    }

    func handleAddToCollection() {
        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: self.entity)
    }

    deinit {
        print("EmailViewController > Deinit")
    }
}
