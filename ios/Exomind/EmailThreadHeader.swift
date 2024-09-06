import UIKit
import SnapKit

class EmailThreadHeader: UIView {
    var label: UILabel!
    var constraintsSet: Bool = false

    override init(frame: CGRect) {
        super.init(frame: frame)
        self.createView()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    func createView() {
        self.label = UILabel()
        self.label.numberOfLines = 0
        self.addSubview(self.label)
    }

    func setupConstraints() {
        self.label.snp.makeConstraints { (make) in
            make.top.equalTo(self.snp.top).offset(10)
            make.left.equalTo(self.snp.left).offset(10)
            make.right.equalTo(self.snp.right).offset(-10)
        }

        self.snp.makeConstraints { (make) in
            make.height.equalTo(self.label.snp.height).offset(30)
            make.width.equalTo(self.superview!.snp.width)
        }
    }

    func load(thread: TraitInstance<Exomind_Base_V1_EmailThread>) {
        self.label.text = thread.message.subject
    }
}
