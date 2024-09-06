import UIKit

class EmailThreadAttachmentTableViewCell: UITableViewCell {
    var attachmentView: AttachmentView?

    func load(attachment: Exomind_Base_V1_EmailAttachment) {
        if let view = attachmentView {
            view.removeFromSuperview()
            attachmentView = nil
        }

        var attachmentName = attachment.name
        if attachmentName == "" {
            attachmentName = "(unnamed attachment)"
        }

        attachmentView = AttachmentView()
        attachmentView!.loadAttachment(attachmentName)
        self.addSubview(attachmentView!)
        attachmentView!.snp.makeConstraints { (make) in
            make.center.equalTo(self.snp.center)
            make.width.equalTo(self.snp.width)
        }
    }
}
