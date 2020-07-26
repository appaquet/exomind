//
//  EmailThreadViewController.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-12-15.
//  Copyright © 2015 Exomind. All rights reserved.
//

import UIKit

class EmailThreadViewController: UITableViewController, EntityTraitViewOld {
    var entityTrait: EntityTraitOld!
    var emails = [EmailFull]()
    var draft: DraftEmailFull?
    
    var opened = [String: Bool]()
    var openedObjectsCell = [String: EmailThreadOpenedTableViewCell]()
    
    var loadTime = Date()
    var headerView: EmailThreadHeader!
    
    func loadEntityTrait(_ entityTrait: EntityTraitOld) {
        if let oldEntityTrait = self.entityTrait, entityTrait.entity.equals(oldEntityTrait.entity) {
            print("EmailThreadViewController > Entity hasn't change, prevent re-rendering")
            return
        }
        
        self.entityTrait = entityTrait
        self.emails = (self.entityTrait.entity.traitsByType[EmailSchema.fullType] ?? [])
            .compactMap { $0 as? EmailFull }
            .sorted(by: { (em1, em2) in
                return em2.receivedDate.isGreaterThan(em1.receivedDate)
            })
        
        self.draft = (self.entityTrait.entity.traitsByType[DraftEmailSchema.fullType] ?? [])
            .compactMap { $0 as? DraftEmailFull }
            .first
        
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
    func sizeTableHeader() {
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
                QuickButtonAction(icon: .check, handler: { [weak self]() -> Void in
                    self?.handleDone()
                })
        ])
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

    func loadData() {
        if self.isViewLoaded {
            self.tableView.reloadData()
            self.headerView.load(entityTrait: self.entityTrait)
            self.sizeTableHeader()
        
            self.perform(#selector(goBottom), with: nil, afterDelay: 0.1)
            self.perform(#selector(goBottom), with: nil, afterDelay: 0.5)
            self.perform(#selector(goBottom), with: nil, afterDelay: 1.0)
        }
    }

    @objc func goBottom() {
        if self.tableView.numberOfSections > 0 {
            let lastSection = max(0, self.tableView.numberOfSections - 1)
            let path = IndexPath(row: 0, section: lastSection)
            self.tableView.scrollToRow(at: path, at: .top, animated: false)
        }
    }
    
    override func numberOfSections(in tableView: UITableView) -> Int {
        let nbDraft = self.draft.map { (d) in return 1 } ?? 0
        return self.emails.count + nbDraft
    }
    
    fileprivate func isDraft(atSection: Int) -> Bool {
        return atSection >= self.emails.count
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        if !self.isDraft(atSection: section) {
            let email = self.emails[section]
            if  self.isOpen(email) {
                let attachmentsCount = email.attachments.count
                if (attachmentsCount > 0) {
                    return 1 + attachmentsCount
                }
            }
            return 1
        } else {
            return 1
        }
    }

    func isOpen(_ email: Email) -> Bool {
        let isUnread = email.unread ?? false
        let isLastEmail = self.emails.last?.id == email.id
        return self.opened[email.id] ?? (isUnread || isLastEmail)
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let section = (indexPath as NSIndexPath).section
        let item = (indexPath as NSIndexPath).item
        
        if self.isDraft(atSection: section), let draft = self.draft {
            var cell: EmailThreadOpenedTableViewCell!
            if let exCell = self.openedObjectsCell["draft"] {
                cell = exCell
            } else {
                cell = (EmailThreadOpenedTableViewCell.loadFromNibNamed("EmailThreadOpenedTableViewCell") as! EmailThreadOpenedTableViewCell)
                self.openedObjectsCell["draft"] = cell
            }
            
            let emailEntityTrait = EntityTraitOld(entity: self.entityTrait.entity, trait: draft)
            cell.load(newEntityTrait: emailEntityTrait, emailIndex: section)
            cell.threadView = self
            return cell
            
        } else {
            let email = self.emails[section]
            if (!self.isOpen(email)) {
                let cell = self.tableView.dequeueReusableCell(withIdentifier: "collapsed", for: indexPath) as! EmailThreadCollapsedTableViewCell
                cell.load(email: email)
                return cell
                
            } else if (email.attachments.count > 0 && item >= 1) {
                let cell = self.tableView.dequeueReusableCell(withIdentifier: "attachments", for: indexPath) as! EmailThreadAttachmentTableViewCell
                let attachmentId = item - 1
                cell.load(attachment: email.attachments[attachmentId])
                return cell
                
            } else {
                var cell: EmailThreadOpenedTableViewCell!
                if let exCell = self.openedObjectsCell[email.id] {
                    cell = exCell
                } else {
                    cell = (EmailThreadOpenedTableViewCell.loadFromNibNamed("EmailThreadOpenedTableViewCell") as! EmailThreadOpenedTableViewCell)
                    self.openedObjectsCell[email.id] = cell
                }
                
                let emailEntityTrait = EntityTraitOld(entity: self.entityTrait.entity, trait: email)
                cell.load(newEntityTrait: emailEntityTrait, emailIndex: section)
                cell.threadView = self
                return cell
            }
        }
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let section = (indexPath as NSIndexPath).section
        if self.isDraft(atSection: section), let draft = self.draft {
            self.openDraftView(draft)
        } else{
            let email = self.emails[section]
            if self.isOpen(email) {
                let isEmailCell = (indexPath as NSIndexPath).item == 0
                if isEmailCell {
                    self.openEmailView(email)
                } else {
                    let attachmentId = (indexPath as NSIndexPath).item - 1
                    if  let attachment = email.attachments[attachmentId] as? FileAttachmentIntegration,
                        let url = EmailsLogic.attachmentUrl(self.entityTrait.entity, email: email, attachment: attachment) {
                        let webView = URLWebViewController(url: URL(string: url)!)
                        (self.navigationController as? NavigationController)?.pushViewController(webView, animated: true)
                    } else {
                        print("Unsupported attachment time \(email.attachments[attachmentId])")
                    }
                }
            } else {
                self.opened[email.id] = true
                self.tableView.reloadData()
            }
        }
    }

    func openEmailView(_ email: EmailFull) {
        let emailEntityTrait = EntityTraitOld(entity: self.entityTrait.entity, trait: email)
        (self.navigationController as? NavigationController)?.pushObject(.entityTraitOld(entityTrait: emailEntityTrait))
    }
    
    func openDraftView(_ draft: DraftEmailFull) {
        let emailEntityTrait = EntityTraitOld(entity: self.entityTrait.entity, trait: draft)
        (self.navigationController as? NavigationController)?.pushObject(.entityTraitOld(entityTrait: emailEntityTrait))
    }

    func handleReply() {
        if let lastEmail = self.emails.last {
            let entityTrait = EntityTraitOld(entity: self.entityTrait.entity, trait: lastEmail)
            EmailsLogic.createReplyEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
                guard   let this = self,
                        let entity = entity,
                        let draft = entity.traitsByType[DraftEmailSchema.fullType]?.first as? DraftEmailFull
                        else { return }
                let entityTrait = EntityTraitOld(entity: entity, trait: draft)
                (this.navigationController as? NavigationController)?.pushObject(.entityTraitOld(entityTrait: entityTrait))
            }
        }
    }

    func handleReplyAll() {
        if let lastEmail = self.emails.last {
            let entityTrait = EntityTraitOld(entity: self.entityTrait.entity, trait: lastEmail)
            EmailsLogic.createReplyAllEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
                guard   let this = self,
                        let entity = entity,
                        let draft = entity.traitsByType[DraftEmailSchema.fullType]?.first as? DraftEmailFull
                        else { return }
                let entityTrait = EntityTraitOld(entity: entity, trait: draft)
                (this.navigationController as? NavigationController)?.pushObject(.entityTraitOld(entityTrait: entityTrait))
            }
        }
    }

    func handleForward() {
        if let lastEmail = self.emails.last {
            let entityTrait = EntityTraitOld(entity: self.entityTrait.entity, trait: lastEmail)
            EmailsLogic.createForwardEmail(entityTrait)?.onProcessed { [weak self] (cmd, entity) -> Void in
                guard   let this = self,
                        let entity = entity,
                        let draft = entity.traitsByType[DraftEmailSchema.fullType]?.first as? DraftEmailFull
                        else { return }
                let entityTrait = EntityTraitOld(entity: entity, trait: draft)
                (this.navigationController as? NavigationController)?.pushObject(.entityTraitOld(entityTrait: entityTrait))
            }
        }
    }

    func handleDone() {
        let inInbox = ExomindDSL.on(self.entityTrait.entity).relations.hasParent(parentId: "inbox")
        if inInbox {
            ExomindDSL.on(self.entityTrait.entity).relations.removeParent(parentId: "inbox")
            let _ = self.navigationController?.popViewController(animated: true)
        }
    }

    func handleAddToCollection() {
        (self.navigationController as? NavigationController)?.showCollectionSelector(forEntity: self.entityTrait.entity)
    }
    
    override func viewWillDisappear(_ animated: Bool) {
        if loadTime.plusSeconds(3).isLessThan(Date()) {
            self.markRead()
        }
    }
    
    func markRead() {
        var modifiedEmails = [EmailBuilder]()
        for email in self.emails {
            if (email.unread ?? true) {
                let builder = EmailBuilder(id: email.id)
                builder.unread = false
                modifiedEmails.append(builder)
            }
        }
        
        if !modifiedEmails.isEmpty {
            ExomindDSL.on(entityTrait.entity).mutate.put(modifiedEmails).execute()
        }
    }

    deinit {
        print("EmailThreadViewController > Deinit")
    }
}
