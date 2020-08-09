import UIKit
import SnapKit
import FontAwesome_swift

class AttachmentView: UIView {
    static let TOTAL_HEIGHT = 45

    override init(frame: CGRect) {
        super.init(frame: frame)
    }

    convenience init() {
        self.init(frame: CGRect.zero)
    }

    required init(coder: NSCoder) {
        fatalError("NSCoding not supported")
    }

    func loadAttachment(_ name: String) {
        let view = UIView()
        view.layer.borderColor = UIColor.gray.cgColor
        view.layer.borderWidth = 1
        view.layer.cornerRadius = 10

        let labelView = UILabel()
        labelView.text = name
        labelView.font = labelView.font.withSize(13)
        labelView.textColor = UIColor.gray
        view.addSubview(labelView)
        labelView.snp.makeConstraints { (make) in
            make.centerY.equalTo(view.snp.centerY)
            make.left.equalTo(view.snp.left).offset(30)
            make.width.equalTo(view.snp.width).offset(-10)
        }

        let img = UIImage.fontAwesomeIcon(name: .paperclip, style: .solid, textColor: UIColor.gray, size: CGSize(width: 20, height: 20))
        let imgView = UIImageView(image: img)
        view.addSubview(imgView)
        imgView.snp.makeConstraints { (make) in
            make.centerY.equalTo(view.snp.centerY)
            make.left.equalTo(view.snp.left).offset(5)
        }

        self.addSubview(view)
        view.snp.makeConstraints { (make) in
            make.top.equalTo(self.snp.top)
            make.bottom.equalTo(self.snp.bottom)
            make.height.equalTo(40)
            make.left.equalTo(self.snp.left).offset(10)
            make.width.equalTo(self.snp.width).offset(-20)
        }
    }
}
