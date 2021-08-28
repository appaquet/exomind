import UIKit

class EmailThreadClosedTableViewCell: UITableViewCell {
    @IBOutlet weak var title: UILabel!
    @IBOutlet weak var date: UILabel!
    @IBOutlet weak var snippet: UILabel!

    func load(draft: TraitInstance<Exomind_Base_V1_DraftEmail>) {
        self.title.text = "Me"
        self.snippet.text = "Draft email"
    }

    func load(email: TraitInstance<Exomind_Base_V1_Email>) {
        self.title.text = Emails.formatContact(email.message.from)
        self.date.text = email.message.receivedDate.date.toShort()
        self.snippet.text = email.message.snippet
    }
}
