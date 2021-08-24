import UIKit

class EmailThreadOpenedTableViewCell: UITableViewCell {
    @IBOutlet weak var webView: EmailBodyWebView!
    @IBOutlet weak var date: UILabel!
    @IBOutlet weak var title: UILabel!
    @IBOutlet weak var to: UILabel!

    private weak var threadView: EmailThreadViewController?

    private var index: Int?
    private var email: TraitInstance<Exomind_Base_V1_Email>?
    private var draft: TraitInstance<Exomind_Base_V1_DraftEmail>?
    private var shouldRender: Bool = false

    private var wasLinkClick: Bool = false

    override func awakeFromNib() {
        super.awakeFromNib()

        self.webView.initialize()
        self.webView.onHeightChange = { [weak self] height in
            self?.threadView?.refreshHeights()
        }

        self.webView.onLoaded = { [weak self] in
            guard let this = self,
                  let emailIndex = this.index else {
                return
            }

            this.threadView?.onEmailWebviewLoaded(emailIndex)
        }
    }

    func load(threadView: EmailThreadViewController, email: TraitInstance<Exomind_Base_V1_Email>, index: Int, renderEmail: Bool) {
        self.threadView = threadView
        if let lastEntityTrait = self.email, email.id == lastEntityTrait.id && email.anyDate == lastEntityTrait.anyDate && email.message.parts == lastEntityTrait.message.parts && renderEmail == shouldRender {
            print("EmailThreadOpenedTableViewCell > Entity didn't change, not refreshing")
            return
        }

        self.email = email
        self.draft = nil
        self.index = index
        self.shouldRender = renderEmail

        self.title.text = Emails.formatContact(email.message.from)
        self.date.text = email.message.receivedDate.date.toShort()
        let emailJoined = (email.message.to + email.message.cc).map {
            Emails.formatContact($0)
        }
        self.to.text = "to \(emailJoined.joined(separator: ", "))"

        if renderEmail {
            let showShortEmail = index > 0
            self.webView.loadEmailEntity(email.message.parts, short: showShortEmail)
            self.webView.onLinkClick = { [weak self] (url) -> Bool in
                self?.wasLinkClick = true
                UIApplication.shared.open(url as URL, options: [:], completionHandler: { (success) in
                })
                return false
            }
        }
    }

    func load(threadView: EmailThreadViewController, draft: TraitInstance<Exomind_Base_V1_DraftEmail>, index: Int, renderEmail: Bool) {
        self.threadView = threadView
        self.draft = draft
        self.email = nil
        self.index = index
        self.shouldRender = renderEmail

        self.title.text = "Draft email"
        let emailJoined = (draft.message.to + draft.message.cc).map {
            Emails.formatContact($0)
        }
        self.to.text = "to \(emailJoined.joined(separator: ", "))"

        if renderEmail {
            let showShortEmail = index > 0
            self.webView.loadEmailEntity(draft.message.parts, short: showShortEmail)
            self.webView.onLinkClick = { (url) -> Bool in
                false
            }
        }
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
            if let email = self.email {
                self.threadView?.openEmailView(email)
            } else if let draft = self.draft {
                self.threadView?.openDraftView(draft)
            }
        }
    }
}

