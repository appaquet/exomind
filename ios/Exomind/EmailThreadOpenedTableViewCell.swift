//
//  EmailThreadOpenedTableViewCell.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-02-29.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit

class EmailThreadOpenedTableViewCell: UITableViewCell {
    @IBOutlet weak var webView: EmailBodyWebView!
    @IBOutlet weak var date: UILabel!
    @IBOutlet weak var title: UILabel!
    @IBOutlet weak var to: UILabel!

    weak var threadView: EmailThreadViewController!
    private var wasLinkClick: Bool = false
    
    private var entityTrait: EntityTraitOld!
    private var email: EmailFull?
    private var draft: DraftEmailFull?
    private var lastTrait: HCTrait?

    override func awakeFromNib() {
        super.awakeFromNib()

        self.webView.initialize()
        self.webView.onHeightChange = { [weak self] height in
            self?.threadView.refreshHeights()
        }
    }
    
    func load(newEntityTrait: EntityTraitOld, emailIndex: Int) {
        if let lastEntityTrait = self.entityTrait, newEntityTrait.trait.equals(lastEntityTrait.trait) {
            print("EmailThreadOpenedTableViewCell > Entity didn't change, not refreshing")
            return
        }
        self.entityTrait = newEntityTrait
        let showShortEmail = emailIndex > 0;
        
        if let email = newEntityTrait.trait as? EmailFull {
            self.email = email
            self.draft = nil
            self.title.text = EmailsLogic.formatContact(email.from)
            self.date.text = email.receivedDate.toShort()
            
            self.webView.loadEmailEntity(newEntityTrait, short: showShortEmail)
            self.webView.onLinkClick = { [weak self] (url) -> Bool in
                self?.wasLinkClick = true
                UIApplication.shared.open(url as URL, options: convertToUIApplicationOpenExternalURLOptionsKeyDictionary([:]), completionHandler: { (sucess) in
                })
                return false
            }
            
            let emailJoined = (email.to + email.cc).map { EmailsLogic.formatContact($0) }
            self.to.text = "to \(emailJoined.joined(separator: ", "))"
            
        } else if let draft = newEntityTrait.trait as? DraftEmailFull {
            self.draft = draft
            self.email = nil
            
            self.title.text = "Draft email"
            self.date.text = " "
            
            self.webView.loadEmailEntity(newEntityTrait, short: showShortEmail)
            self.webView.onLinkClick = { (url) -> Bool in
                return false
            }
            
            let emailJoined = (draft.to + draft.cc).map { EmailsLogic.formatContact($0) }
            self.to.text = "to \(emailJoined.joined(separator: ", "))"
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

// Helper function inserted by Swift 4.2 migrator.
fileprivate func convertToUIApplicationOpenExternalURLOptionsKeyDictionary(_ input: [String: Any]) -> [UIApplication.OpenExternalURLOptionsKey: Any] {
	return Dictionary(uniqueKeysWithValues: input.map { key, value in (UIApplication.OpenExternalURLOptionsKey(rawValue: key), value)})
}
