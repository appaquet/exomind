//
//  EmailThreadAttachmentTableViewCell.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-02-29.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit

class EmailThreadAttachmentTableViewCell: UITableViewCell {
    var attachmentView: AttachmentView?
    
    func load(attachment: FileAttachment) {
        if let view = attachmentView {
            view.removeFromSuperview()
            attachmentView = nil
        }
        
        attachmentView = AttachmentView()
        attachmentView!.loadAttachment(attachment.name ?? "(unnamed attachment)")
        self.addSubview(attachmentView!)
        attachmentView!.snp.makeConstraints { (make) in
            make.center.equalTo(self.snp.center)
            make.width.equalTo(self.snp.width)
        }
        
    }
}
