import UIKit

class EmailThreadOpenedTableViewCell: UITableViewCell {
    @IBOutlet weak var webView: EmailBodyWebView!
    @IBOutlet weak var date: UILabel!
    @IBOutlet weak var title: UILabel!
    @IBOutlet weak var to: UILabel!

    weak var threadView: EmailThreadViewController!
    private var wasLinkClick: Bool = false

    private var emailIndex: Int?
    private var email: TraitInstance<Exomind_Base_Email>?
    private var draft: TraitInstance<Exomind_Base_DraftEmail>?
    private var lastTrait: HCTrait?

    override func awakeFromNib() {
        super.awakeFromNib()

        self.webView.initialize()
        self.webView.onHeightChange = { [weak self] height in
            self?.threadView.refreshHeights()
        }

        self.webView.onLoaded = { [weak self] in
            guard let this = self,
                  let emailIndex = this.emailIndex else { return }

            this.threadView.onEmailWebviewLoaded(emailIndex)
        }
    }

    func load(email: TraitInstance<Exomind_Base_Email>, emailIndex: Int) {
        // TODO: put back once we know if it was unloaded before OR should it be in email thread view controller
//        if let lastEntityTrait = self.email, email.id == lastEntityTrait.id && email.anyDate == lastEntityTrait.anyDate {
//            print("EmailThreadOpenedTableViewCell > Entity didn't change, not refreshing")
//            return
//        }

        let showShortEmail = emailIndex > 0
        self.email = email
        self.draft = nil
        self.title.text = EmailsLogic.formatContact(email.message.from)
        self.date.text = email.message.receivedDate.date.toShort()

        self.emailIndex = emailIndex

        self.webView.loadEmailEntity(email.message.parts, short: showShortEmail)
        self.webView.onLinkClick = { [weak self] (url) -> Bool in
            self?.wasLinkClick = true
            UIApplication.shared.open(url as URL, options: convertToUIApplicationOpenExternalURLOptionsKeyDictionary([:]), completionHandler: { (sucess) in
            })
            return false
        }

        let emailJoined = (email.message.to + email.message.cc).map {
            EmailsLogic.formatContact($0)
        }
        self.to.text = "to \(emailJoined.joined(separator: ", "))"
    }

    func load(draft: TraitInstance<Exomind_Base_DraftEmail>, emailIndex: Int) {
        self.draft = draft
        self.email = nil

        self.title.text = "Draft email"
        self.date.text = " "

//        let showShortEmail = emailIndex > 0
//        self.webView.loadEmailEntity(email, short: showShortEmail)
//        self.webView.onLinkClick = { (url) -> Bool in
//            return false
//        }

        let emailJoined = (draft.message.to + draft.message.cc).map {
            EmailsLogic.formatContact($0)
        }
        self.to.text = "to \(emailJoined.joined(separator: ", "))"
    }

    override func gestureRecognizer(_ gestureRecognizer: UIGestureRecognizer, shouldRecognizeSimultaneouslyWith otherGestureRecognizer: UIGestureRecognizer) -> Bool {
        // from http://stackoverflow.com/questions/8497815/detect-single-tap-in-uiwebview-but-still-support-text-selection-and-links
        if let _ = otherGestureRecognizer as? UITapGestureRecognizer {
            self.wasLinkClick = false
            NSObject.cancelPreviousPerformRequests(withTarget: self)
            self.perform(#selector(maybeOpenEmailView), with: nil, afterDelay: 0.5)
        }

        return false
    }

    @objc func maybeOpenEmailView() {
        if !self.wasLinkClick {
//            if let email = self.email {
//                self.threadView?.openEmailView(email)
//            } else if let draft = self.draft {
//                self.threadView?.openDraftView(draft)
//            }
        }
    }
}

// Helper function inserted by Swift 4.2 migrator.
fileprivate func convertToUIApplicationOpenExternalURLOptionsKeyDictionary(_ input: [String: Any]) -> [UIApplication.OpenExternalURLOptionsKey: Any] {
    return Dictionary(uniqueKeysWithValues: input.map { key, value in
        (UIApplication.OpenExternalURLOptionsKey(rawValue: key), value)
    })
}
