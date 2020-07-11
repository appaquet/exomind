//
//  EmailThreadCollapsedTableViewCell.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-02-29.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit

class EmailThreadCollapsedTableViewCell: UITableViewCell {
    @IBOutlet weak var title: UILabel!
    @IBOutlet weak var date: UILabel!
    @IBOutlet weak var snippet: UILabel!

    func load(draft: DraftEmail) {
        self.title.text = "Me"
        self.snippet.text = "Draft email"
    }
    
    func load(email: Email) {
        self.title.text = EmailsLogic.formatContact(email.from)
        self.date.text = email.receivedDate.toShort()
        self.snippet.text = email.snippet
    }
}
