import UIKit
import Exocore

class EmailThreadViewController: UITableViewController, EntityTraitView {
    private var entity: EntityExt!

    private var thread: TraitInstance<Exomind_Base_V1_EmailThread>!
    private var emails = [TraitInstance<Exomind_Base_V1_Email>]()
    private var draft: TraitInstance<Exomind_Base_V1_DraftEmail>?
    private var firstNonRead: Int = 0

    private var loadedEmails: [Bool] = []
    private var unreadEmails: Dictionary<TraitId, TraitInstance<Exomind_Base_V1_Unread>> = Dictionary()

    private var opened = [String: Bool]()
    private var openedObjectsCell = [String: EmailThreadOpenedTableViewCell]()

    private var navigatedFirstNonRead = false
    private var loadTime = Date()
    private var headerView: EmailThreadHeader!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance, fullEntity: Bool) {
        self.entity = entity
        self.thread = entity.traitOfType(Exomind_Base_V1_EmailThread.self)!

        if self.unreadEmails.isEmpty {
            // only update unread flags if it's the first time so that we don't update based on our own mark as read
            self.unreadEmails = Dictionary(uniqueKeysWithValues: entity.traitsOfType(Exomind_Base_V1_Unread.self).map { unread in
                (unread.message.entity.traitID, unread)
            })
        }

        self.emails = entity
                .traitsOfType(Exomind_Base_V1_Email.self)
                .sorted(by: { (em1, em2) in
                    em1.message.receivedDate.date.isLessThan(em2.message.receivedDate.date)
                })
        self.firstNonRead = self.emails.firstIndex(where: { self.unreadEmails[$0.id] != nil }) ?? self.emails.count - 1
        
        // unread emails are considered loaded since we use to sequentially load them
        self.loadedEmails = self.emails.map({ self.unreadEmails[$0.id] == nil })

        self.draft = entity.traitOfType(Exomind_Base_V1_DraftEmail.self)

        if self.draft != nil {
            self.loadedEmails.append(false)
        }

        self.loadData()
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        self.tableView.delegate = self
        self.tableView.rowHeight = UITableView.automaticDimension
        self.tableView.estimatedRowHeight = 49

        self.headerView = EmailThreadHeader()
        self.sizeTableHeader()

        self.loadData()
    }

    // http://stackoverflow.com/questions/19005446/table-header-view-height-is-wrong-when-using-auto-layout-ib-and-font-sizes
    private func sizeTableHeader() {
        self.headerView.setNeedsLayout()
        self.headerView.layoutIfNeeded()
        var headerFrame = self.tableView.frame
        headerFrame.size.height = self.headerView.systemLayoutSizeFitting(UIView.layoutFittingCompressedSize).height

        self.headerView.frame = headerFrame
        self.tableView.tableHeaderView = nil
        self.tableView.tableHeaderView = self.headerView
        self.headerView.setupConstraints()
    }

    override func viewWillAppear(_ animated: Bool) {
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
                QuickButtonAction(icon: .check, handler: { [weak self]() -> Void in
                    self?.handleDone()
                })
            ])
        }
    }

    func refreshHeights() {
        print("EmailThreadViewController > Refreshing heights")
        // http://stackoverflow.com/questions/9309929/i-do-not-want-animation-in-the-begin-updates-end-updates-block-for-uitableview
        UIView.setAnimationsEnabled(false)
        let offset = self.tableView.contentOffset
        self.tableView.beginUpdates()
        self.tableView.endUpdates()
        self.tableView.contentOffset = offset
        UIView.setAnimationsEnabled(true)
    }

    func onEmailWebviewLoaded(_ emailIndex: Int) {
        print("EmailThreadViewController > Email webview loaded \(emailIndex)")
        if emailIndex < self.loadedEmails.count {
            self.loadedEmails[emailIndex] = true
            self.loadData()
        }
    }

    func loadData() {
        self.tableView.reloadData()
        self.headerView.load(thread: self.thread)
        self.sizeTableHeader()

        self.perform(#selector(navigateFirstNonRead), with: nil, afterDelay: 0.1)
        self.perform(#selector(navigateFirstNonRead), with: nil, afterDelay: 0.5)
        self.perform(#selector(navigateFirstNonRead), with: nil, afterDelay: 1.0)
    }

    @objc private func navigateFirstNonRead() {
        if self.navigatedFirstNonRead {
            return
        }

        self.navigatedFirstNonRead = true
        if self.firstNonRead > 0 && self.tableView.numberOfSections > 1 {
            let path = IndexPath(row: 0, section: self.firstNonRead)
            self.tableView.scrollToRow(at: path, at: .top, animated: false)
        }

        self.perform(#selector(markRead), with: nil, afterDelay: 3.0) // sync delay with `email-thread.tsx`
    }

    override func numberOfSections(in tableView: UITableView) -> Int {
        let nbDraft = self.draft.map { (d) in
            1
        } ?? 0
        return self.emails.count + nbDraft
    }

    private func isDraft(atSection: Int) -> Bool {
        atSection >= self.emails.count
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        if !self.isDraft(atSection: section) {
            let email = self.emails[section]
            if self.isOpen(email) {
                let attachmentsCount = email.message.attachments.count
                if (attachmentsCount > 0) {
                    return 1 + attachmentsCount
                }
            }
            return 1
        } else {
            return 1
        }
    }

    private func isOpen(_ email: TraitInstance<Exomind_Base_V1_Email>) -> Bool {
        let isUnread = self.unreadEmails[email.id] != nil
        let isLastEmail = self.emails.last?.id == email.id
        return self.opened[email.id] ?? (isUnread || isLastEmail)
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let section = (indexPath as NSIndexPath).section
        let item = (indexPath as NSIndexPath).item

        // this is used to prevent rendering all emails at once, but only render in sequence
        let showRenderEmail = section == 0 || (section < self.loadedEmails.count && self.loadedEmails[section - 1])

        if self.isDraft(atSection: section), let draft = self.draft {
            var cell: EmailThreadOpenedTableViewCell!
            if let exCell = self.openedObjectsCell["draft"] {
                cell = exCell
            } else {
                cell = (EmailThreadOpenedTableViewCell.loadFromNibNamed("EmailThreadOpenedTableViewCell") as! EmailThreadOpenedTableViewCell)
                self.openedObjectsCell["draft"] = cell
            }

            cell.load(threadView: self, draft: draft, index: section, renderEmail: showRenderEmail)
            return cell

        } else {
            let email = self.emails[section]
            if (!self.isOpen(email)) {
                let cell = self.tableView.dequeueReusableCell(withIdentifier: "collapsed", for: indexPath) as! EmailThreadClosedTableViewCell
                cell.load(email: email)
                return cell

            } else if (email.message.attachments.count > 0 && item >= 1) {
                let cell = self.tableView.dequeueReusableCell(withIdentifier: "attachments", for: indexPath) as! EmailThreadAttachmentTableViewCell
                let attachmentId = item - 1
                cell.load(attachment: email.message.attachments[attachmentId])
                return cell

            } else {
                var cell: EmailThreadOpenedTableViewCell!
                if let exCell = self.openedObjectsCell[email.id] {
                    cell = exCell
                } else {
                    cell = (EmailThreadOpenedTableViewCell.loadFromNibNamed("EmailThreadOpenedTableViewCell") as! EmailThreadOpenedTableViewCell)
                    self.openedObjectsCell[email.id] = cell
                }

                cell.load(threadView: self, email: email, index: section, renderEmail: showRenderEmail)
                return cell
            }
        }
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let section = (indexPath as NSIndexPath).section
        if self.isDraft(atSection: section), let draft = self.draft {
            self.openDraftView(draft)
        } else {
            let email = self.emails[section]
            if self.isOpen(email) {
                let isEmailCell = (indexPath as NSIndexPath).item == 0
                if isEmailCell {
                    self.openEmailView(email)
                } else {
                    // TODO:
//                    let attachmentId = (indexPath as NSIndexPath).item - 1
//                    if let attachment = email.message.attachments[attachmentId] as? FileAttachmentIntegration,
//                       let url = EmailsLogic.attachmentUrl(self.entity.entity, email: email, attachment: attachment) {
//                        let webView = URLWebViewController(url: URL(string: url)!)
//                        (self.navigationController as? NavigationController)?.pushViewController(webView, animated: true)
//                    } else {
//                        print("Unsupported attachment time \(email.message.attachments[attachmentId])")
//                    }
                }
            } else {
                self.opened[email.id] = true
                self.tableView.reloadData()
            }
        }
    }

    func openEmailView(_ email: TraitInstance<Exomind_Base_V1_Email>) {
        if let anyTrait = email.toAny() {
            (self.navigationController as? NavigationController)?.pushObject(.entityTrait(entityTrait: anyTrait))
        }
    }

    func openDraftView(_ draft: TraitInstance<Exomind_Base_V1_DraftEmail>) {
        if let anyTrait = draft.toAny() {
            (self.navigationController as? NavigationController)?.pushObject(.entityTrait(entityTrait: anyTrait))
        }
    }

    private func handleReply() {
//        if let lastEmail = self.emails.last {
//            let entityTrait = EntityTraitOld(entity: self.entityTrait.entity, trait: lastEmail)
//            EmailsLogic.createReplyEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
//                guard   let this = self,
//                        let entity = entity,
//                        let draft = entity.traitsByType[DraftEmailSchema.fullType]?.first as? DraftEmailFull
//                        else { return }
//                let entityTrait = EntityTraitOld(entity: entity, trait: draft)
//                (this.navigationController as? NavigationController)?.pushObject(.entityTraitOld(entityTrait: entityTrait))
//            }
//        }
    }

    private func handleReplyAll() {
//        if let lastEmail = self.emails.last {
//            let entityTrait = EntityTraitOld(entity: self.entityTrait.entity, trait: lastEmail)
//            EmailsLogic.createReplyAllEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
//                guard   let this = self,
//                        let entity = entity,
//                        let draft = entity.traitsByType[DraftEmailSchema.fullType]?.first as? DraftEmailFull
//                        else { return }
//                let entityTrait = EntityTraitOld(entity: entity, trait: draft)
//                (this.navigationController as? NavigationController)?.pushObject(.entityTraitOld(entityTrait: entityTrait))
//            }
//        }
    }

    private func handleForward() {
//        if let lastEmail = self.emails.last {
//            let entityTrait = EntityTraitOld(entity: self.entityTrait.entity, trait: lastEmail)
//            EmailsLogic.createForwardEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
//                guard   let this = self,
//                        let entity = entity,
//                        let draft = entity.traitsByType[DraftEmailSchema.fullType]?.first as? DraftEmailFull
//                        else { return }
//                let entityTrait = EntityTraitOld(entity: entity, trait: draft)
//                (this.navigationController as? NavigationController)?.pushObject(.entityTraitOld(entityTrait: entityTrait))
//            }
//        }
    }

    private func handleDone() {
        let inInbox = Collections.hasParent(entity: self.entity, parentId: "inbox")
        if inInbox {
            Commands.removeFromParent(entity: self.entity, parentId: "inbox")
            let _ = self.navigationController?.popViewController(animated: true)
        }
    }

    private func handleAddToCollection() {
        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: self.entity)
    }

    @objc private func markRead() {
        if self.unreadEmails.isEmpty {
            return
        }

        var mutation = MutationBuilder.updateEntity(entityId: self.entity.id)
        for email in self.emails {
            if let unread = self.unreadEmails[email.id] {
                mutation = mutation.deleteTrait(traitId: unread.id)
            }
        }

        ExocoreClient.store.mutate(mutation: mutation.build())
    }

    deinit {
        print("EmailThreadViewController > Deinit")
    }
}
