import UIKit
import FontAwesome_swift

class GridIconsView: UIView {
    var items = [GridIconsViewItem]()
    var squareSize = 90
    var imageSize = 50
    var squareOffset = 10
    var squarePerRow = 3
    var squareColor = UIColor.white.withAlphaComponent(0.7)
    var squareBorderWidth = CGFloat(1)

    override init(frame: CGRect) {
        super.init(frame: frame)
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    convenience init(items: [GridIconsViewItem]) {
        self.init(frame: CGRect.zero)
        self.items = items
    }

    func initView() {
        let nbRow = Int(ceil(Double(items.count) / Double(squarePerRow)))

        let centerView = UIView()
        self.addSubview(centerView)
        centerView.snp.makeConstraints { (make) in
            make.center.equalTo(self.snp.center)
            make.width.equalTo(squareSize * squarePerRow + squareOffset * (squarePerRow - 1))
            make.height.equalTo(squareSize * nbRow + squareOffset * (nbRow - 1))
        }

        var choicesViews = [UIView]()
        for (i, choice) in self.items.enumerated() {
            let choiceView = UIView()
            choiceView.layer.borderWidth = squareBorderWidth
            choiceView.layer.borderColor = squareColor.cgColor
            choiceView.tag = i
            centerView.addSubview(choiceView)
            choicesViews.append(choiceView)

            let colPos = i % squarePerRow
            let rowPos = i / squarePerRow
            choiceView.snp.makeConstraints({ (make) in
                if (colPos == 0) {
                    make.left.equalTo(centerView.snp.left)
                } else {
                    make.left.equalTo(choicesViews[i - 1].snp.right).offset(squareOffset)
                }
                if (rowPos == 0) {
                    make.top.equalTo(centerView.snp.top)
                } else {
                    make.top.equalTo(choicesViews[i - squarePerRow].snp.bottom).offset(squareOffset)
                }
                make.width.equalTo(squareSize)
                make.height.equalTo(squareSize)
            })

            let choiceImg = UIImageView(image: UIImage.fontAwesomeIcon(name: choice.icon, style: .solid, textColor: squareColor, size: CGSize(width: imageSize, height: imageSize)))
            choiceView.addSubview(choiceImg)
            choiceImg.snp.makeConstraints { (make) in
                make.centerX.equalTo(choiceView.snp.centerX)
                make.centerY.equalTo(choiceView.snp.centerY).offset(-7)
            }

            let choiceTxt = UILabel()
            choiceView.addSubview(choiceTxt)
            choiceTxt.textColor = squareColor
            choiceTxt.text = choice.label
            choiceTxt.font = choiceTxt.font.withSize(12)
            choiceTxt.textAlignment = .center
            choiceTxt.snp.makeConstraints { (make) in
                make.centerX.equalTo(choiceView.snp.centerX)
                make.top.equalTo(choiceImg.snp.bottom).offset(3)
                make.width.equalTo(choiceView.snp.width).offset(-5)
            }

            let tapRecognizer = UITapGestureRecognizer(target: self, action: #selector(handleChoiceSelection))
            choiceView.addGestureRecognizer(tapRecognizer)
        }
    }

    @objc func handleChoiceSelection(_ sender: UITapGestureRecognizer) {
        if let tag = sender.view?.tag {
            let item = self.items[tag]
            item.callback(item)
        }
    }
}

class GridIconsViewItem {
    let label: String
    let icon: FontAwesome
    let callback: (GridIconsViewItem) -> ()

    init(label: String, icon: FontAwesome, callback: @escaping (GridIconsViewItem) -> ()) {
        self.label = label
        self.icon = icon
        self.callback = callback
    }
}
