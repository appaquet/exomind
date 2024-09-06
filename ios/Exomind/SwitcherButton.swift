import UIKit
import FontAwesome_swift

class SwitcherButton: UIView {
    fileprivate var bgView: UIView!
    fileprivate var actions: [SwitcherButtonAction] = []
    fileprivate var actionsView: [UIView] = []
    fileprivate let buttonWidth = CGFloat(50)
    fileprivate let buttonHeight = CGFloat(30)

    fileprivate var widthConstraint: NSLayoutConstraint!
    fileprivate var heightConstraint: NSLayoutConstraint!

    override init(frame: CGRect) {
        super.init(frame: frame)
        self.createView()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    func createView() {
        self.autoresizesSubviews = false
        self.translatesAutoresizingMaskIntoConstraints = false

        // make the view round and clip buttons that overflow inside
        self.layer.cornerRadius = 5
        self.layer.borderColor = Stylesheet.switcherButtonBorderFg.cgColor
        self.layer.borderWidth = 1
        self.clipsToBounds = true

        self.widthConstraint = NSLayoutConstraint(item: self, attribute: .width, relatedBy: .equal, toItem: nil, attribute: .width, multiplier: 0, constant: 0)
        self.heightConstraint = NSLayoutConstraint(item: self, attribute: .height, relatedBy: .equal, toItem: nil, attribute: .height, multiplier: 0, constant: buttonHeight)
        self.addConstraints([
            self.widthConstraint,
            self.heightConstraint
        ])
    }

    func setActions(_ actions: [SwitcherButtonAction]) {
        self.actions = actions
        self.actionsView.forEach {
            $0.removeFromSuperview()
        }
        self.actionsView = actions.enumerated().map {
            (i, action) in
            let bg = UIView()
            bg.frame = CGRect(x: 0, y: 0, width: buttonWidth, height: buttonHeight)
            bg.autoresizesSubviews = true
            bg.layer.borderColor = Stylesheet.switcherButtonBorderFg.cgColor
            bg.layer.borderWidth = 0.5

            var imgColor: UIColor!
            if (action.active) {
                bg.backgroundColor = Stylesheet.switcherButtonActiveBg
                imgColor = Stylesheet.switcherButtonInactiveBg
            } else {
                bg.backgroundColor = Stylesheet.switcherButtonInactiveBg
                imgColor = Stylesheet.switcherButtonActiveBg
            }

            let button = UIButton()
            let img = UIImage.fontAwesomeIcon(name: action.icon, style: .solid, textColor: imgColor, size: CGSize(width: 20, height: 20))
            button.setImage(img, for: UIControl.State())
            button.autoresizingMask = [.flexibleWidth, .flexibleHeight]
            button.frame = bg.frame
            button.tag = i
            button.addTarget(self, action: #selector(handleButtonClick), for: .touchUpInside)

            bg.addSubview(button)

            return bg
        }
        for view in actionsView {
            self.addSubview(view)
        }

        let width = buttonWidth * CGFloat(actions.count)
        self.widthConstraint.constant = width
    }

    @objc func handleButtonClick(_ button: UIButton) {
        self.actions[button.tag].callback?()
    }

    override func layoutSubviews() {
        for (i, view) in actionsView.enumerated() {
            view.frame = CGRect(x: CGFloat(i) * buttonWidth, y: view.frame.origin.y, width: view.frame.width, height: view.frame.height)
        }
    }
}

class SwitcherButtonAction {
    let callback: (() -> Void)?
    let icon: FontAwesome
    let active: Bool

    init(icon: FontAwesome, active: Bool, callback: (() -> Void)?) {
        self.icon = icon
        self.callback = callback
        self.active = active
    }
}
