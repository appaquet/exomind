import UIKit

class LabelledFieldView: UIView {
    var label: String!
    var fieldView: UIView!
    var border: Bool = true
    var betweenPadding: CGFloat = 5

    override init(frame: CGRect) {
        super.init(frame: frame)
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    convenience init(label: String, fieldView: UIView, betweenPadding: CGFloat = 5) {
        self.init(frame: CGRect.zero)
        self.label = label
        self.fieldView = fieldView
        self.betweenPadding = betweenPadding
        self.createView()
    }

    func createView() {
        if (self.border) {
            self.addBorderBottomView()
        }

        let labelView = UILabel()
        labelView.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        labelView.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        labelView.text = "\(self.label!):"
        labelView.font = UIFont.systemFont(ofSize: 14)
        labelView.textColor = UIColor.lightGray
        self.addSubview(labelView)
        labelView.snp.makeConstraints { (make) in
            make.left.equalTo(self.snp.left).offset(10)
            make.centerY.equalTo(self.snp.centerY).offset(-1) // fixes mis-alignment of text
        }

        self.addSubview(fieldView)
        fieldView.snp.makeConstraints { (make) in
            make.left.equalTo(labelView.snp.right).offset(10)
            make.right.equalTo(self.snp.right).offset(10)
            make.top.equalTo(self.snp.top).offset(betweenPadding)
            make.bottom.equalTo(self.snp.bottom).offset(-betweenPadding - 1) // -1 for border
        }
        fieldView.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)

        self.snp.makeConstraints { (make) in
            make.height.greaterThanOrEqualTo(labelView.snp.height).offset(betweenPadding * 2)
        }
    }
}
