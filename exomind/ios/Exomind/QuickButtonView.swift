import UIKit
import FontAwesome_swift

class QuickButtonView: UIView {
    fileprivate var panRecognizer: UIPanGestureRecognizer!
    fileprivate var mainTapRecognizer: UITapGestureRecognizer!
    fileprivate var mainButtonView: UIView!
    fileprivate var outerRecognizer: UIGestureRecognizer!

    fileprivate var actionsViews: [QuickButtonView] = []
    fileprivate var actions: [QuickButtonAction] = []
    fileprivate var lastClickedAction: QuickButtonAction?

    fileprivate var widthConstraint: NSLayoutConstraint!
    fileprivate var heightConstraint: NSLayoutConstraint!
    fileprivate var bottomConstraint: NSLayoutConstraint!

    override init(frame: CGRect) {
        super.init(frame: frame)
        self.createMainButton()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    func isOpen() -> Bool {
        return !self.actionsViews.isEmpty
    }

    fileprivate func createMainButton() {
        self.translatesAutoresizingMaskIntoConstraints = false

        self.mainButtonView = UIView()
        self.mainButtonView.frame = self.frame
        self.mainButtonView.layer.cornerRadius = Stylesheet.quickButtonSize / 2
        self.mainButtonView.backgroundColor = Stylesheet.quickButtonBg
        self.mainButtonView.translatesAutoresizingMaskIntoConstraints = false
        self.mainButtonView.alpha = Stylesheet.quickButtonAlphaClosed

        let img = UIImage.fontAwesomeIcon(name: .bolt, style: .solid, textColor: Stylesheet.quickButtonFg, size: CGSize(width: Stylesheet.quickButtonSize * 2, height: Stylesheet.quickButtonSize * 2))
        let imgView = UIImageView(image: img)
        imgView.frame = self.frame
        imgView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        mainButtonView.addSubview(imgView)

        self.addSubview(self.mainButtonView)

        self.addConstraints([
            NSLayoutConstraint(item: self.mainButtonView!, attribute: .centerX, relatedBy: .equal, toItem: self, attribute: .centerX, multiplier: 1, constant: 0),
            NSLayoutConstraint(item: self.mainButtonView!, attribute: .bottom, relatedBy: .equal, toItem: self, attribute: .bottom, multiplier: 1, constant: 0),
            NSLayoutConstraint(item: self.mainButtonView!, attribute: .width, relatedBy: .equal, toItem: nil, attribute: .width, multiplier: 0, constant: Stylesheet.quickButtonSize),
            NSLayoutConstraint(item: self.mainButtonView!, attribute: .height, relatedBy: .equal, toItem: nil, attribute: .height, multiplier: 0, constant: Stylesheet.quickButtonSize),
        ])

        self.panRecognizer = UIPanGestureRecognizer(target: self, action: #selector(handleMainPan))
        self.mainButtonView.addGestureRecognizer(self.panRecognizer)

        self.mainTapRecognizer = UITapGestureRecognizer(target: self, action: #selector(toggle))
        self.mainButtonView.addGestureRecognizer(self.mainTapRecognizer)
    }

    @objc func handleMainPan(_ sender: UIPanGestureRecognizer) {
        let minDistance = CGFloat(100)
        let translation = self.panRecognizer.translation(in: self)
        let angle = atan2(-translation.y, translation.x)
        let dist = sqrt(pow(translation.y, 2) + pow(translation.x, 2))

        if (sender.state == .ended && dist > minDistance) {
            let index = round((1 - abs(angle) / 3.14159) * CGFloat(self.actions.count - 1))
            let action = self.actions[Int(index)]
            action.handler?()
        }
    }

    func addToView(_ containerView: UIView, bottomMargin: CGFloat = 10) {
        containerView.addSubview(self)
        self.bottomConstraint = NSLayoutConstraint(item: self, attribute: .bottom, relatedBy: .equal, toItem: containerView, attribute: .bottom, multiplier: 1, constant: -bottomMargin)
        containerView.addConstraint(self.bottomConstraint)
        containerView.addConstraint(NSLayoutConstraint(item: self, attribute: .centerX, relatedBy: .equal, toItem: containerView, attribute: .centerX, multiplier: 1, constant: 0))
        self.widthConstraint = NSLayoutConstraint(item: self, attribute: .width, relatedBy: .equal, toItem: containerView, attribute: .width, multiplier: 0, constant: Stylesheet.quickButtonSize)
        self.heightConstraint = NSLayoutConstraint(item: self, attribute: .height, relatedBy: .equal, toItem: containerView, attribute: .height, multiplier: 0, constant: Stylesheet.quickButtonSize)
        containerView.addConstraints([self.widthConstraint, self.heightConstraint])

        self.outerRecognizer = UITapGestureRecognizer(target: self, action: #selector(close))
        self.addGestureRecognizer(outerRecognizer)
    }

    func setBottomMargin(_ bottomMargin: CGFloat) {
        self.bottomConstraint.constant = -bottomMargin
    }

    @objc func toggle() {
        if (!self.isOpen()) {
            self.open()
        } else {
            self.close()
        }
    }

    func setActions(_ actions: [QuickButtonAction]) {
        self.close()
        self.actions = actions
        self.lastClickedAction = nil
    }

    func open() {
        if (self.isOpen()) {
            return
        }

        self.widthConstraint.constant = 1000
        self.heightConstraint.constant = 1000

        self.mainButtonView.alpha = Stylesheet.quickButtonAlphaOpened
        self.actionsViews = []
        for (i, action) in self.actions.enumerated() {
            let actionButton = UIView()
            actionButton.frame = CGRect(x: 0, y: 0, width: Stylesheet.quickSecondarySize, height: Stylesheet.quickSecondarySize)
            actionButton.center = self.mainButtonView.center
            actionButton.layer.cornerRadius = Stylesheet.quickSecondarySize / 2
            actionButton.backgroundColor = Stylesheet.quickSecondaryBg
            actionButton.translatesAutoresizingMaskIntoConstraints = false
            actionButton.autoresizesSubviews = true
            actionButton.tag = i

            let img = UIImage.fontAwesomeIcon(name: action.icon, style: .solid, textColor: Stylesheet.quickSecondaryFg, size: CGSize(width: Stylesheet.quickSecondaryImgSize * 2, height: Stylesheet.quickSecondaryImgSize * 2))
            let imgView = UIImageView(image: img)
            imgView.frame = CGRect(x: 0, y: 0, width: Stylesheet.quickSecondaryImgSize, height: Stylesheet.quickSecondaryImgSize)
            imgView.center = CGPoint(x: Stylesheet.quickSecondarySize / 2, y: Stylesheet.quickSecondarySize / 2)
            imgView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
            actionButton.addSubview(imgView)

            self.addSubview(actionButton)
            self.sendSubviewToBack(actionButton)

            // set initial constraint
            let constraintX = NSLayoutConstraint(item: actionButton, attribute: .centerX, relatedBy: .equal, toItem: mainButtonView, attribute: .centerX, multiplier: 1, constant: 0)
            let constraintY = NSLayoutConstraint(item: actionButton, attribute: .centerY, relatedBy: .equal, toItem: mainButtonView, attribute: .centerY, multiplier: 1, constant: 0)
            let constraintWidth = NSLayoutConstraint(item: actionButton, attribute: .width, relatedBy: .equal, toItem: nil, attribute: .width, multiplier: 0, constant: Stylesheet.quickSecondarySize)
            let constraintHeight = NSLayoutConstraint(item: actionButton, attribute: .height, relatedBy: .equal, toItem: nil, attribute: .height, multiplier: 0, constant: Stylesheet.quickSecondarySize)
            let actionConstraints = [constraintX, constraintY, constraintWidth, constraintHeight]
            self.addConstraints(actionConstraints)
            self.layoutIfNeeded()

            let tapRecognizer = UITapGestureRecognizer(target: self, action: #selector(handleActionClick))
            actionButton.addGestureRecognizer(tapRecognizer)

            self.actionsViews.append(QuickButtonView(view: actionButton, constraints: actionConstraints, gesture: tapRecognizer))

            // change constraints and then animate them
            let actionsCount = (self.actions.count == 1) ? Double(1) : Double(self.actions.count - 1)

            constraintX.constant = -CGFloat(cos(Double(i) / actionsCount * Double.pi) * Stylesheet.quickSecondaryDistance)
            constraintY.constant = -CGFloat(sin(Double(i) / actionsCount * Double.pi) * Stylesheet.quickSecondaryDistance)
            UIView.animate(withDuration: 0.1, animations: {
                self.layoutIfNeeded()
            })
        }
    }

    @objc func handleActionClick(_ sender: UITapGestureRecognizer) {
        if let index = sender.view?.tag {
            self.close()

            // calling handler after a delay is needed, otherwise it triggers a bug if the handler is dimissing the view controller (or popping on nav controller)
            self.lastClickedAction = self.actions[index]
            self.perform(#selector(callHandler), with: nil, afterDelay: 0.01)
        }
    }

    @objc func callHandler() {
        self.lastClickedAction?.handler?()
    }

    @objc func close() {
        if (!self.isOpen()) {
            return
        }

        self.widthConstraint.constant = Stylesheet.quickButtonSize
        self.heightConstraint.constant = Stylesheet.quickButtonSize
        self.mainButtonView.alpha = Stylesheet.quickButtonAlphaClosed
        actionsViews.forEach {
            (view) -> () in
            view.view.removeGestureRecognizer(view.gesture)
            view.view.removeFromSuperview()
            self.removeConstraints(view.constraints)
        }
        actionsViews.removeAll()
    }

    deinit {
        print("QuickButtonView > Deinit")
    }

    fileprivate class QuickButtonView {
        let view: UIView
        let constraints: [NSLayoutConstraint]
        let gesture: UITapGestureRecognizer

        init(view: UIView, constraints: [NSLayoutConstraint], gesture: UITapGestureRecognizer) {
            self.view = view
            self.constraints = constraints
            self.gesture = gesture
        }
    }

}

class QuickButtonAction {
    let icon: FontAwesome
    let handler: (() -> Void)?

    init(icon: FontAwesome, handler: (() -> Void)?) {
        self.icon = icon
        self.handler = handler
    }
}
