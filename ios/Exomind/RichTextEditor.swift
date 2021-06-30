import UIKit
import SwiftyJSON
import FontAwesome_swift
import SnapKit

/*
 TODO:
    - Placing cursor on empty line doesn't scroll
 */

class RichTextEditor: UIViewController {
    fileprivate weak var outerScroll: UIScrollView?
    fileprivate var webview: RichTextEditorWebView!

    private var keyboardRect = CGRect(x: 0, y: 500, width: 0, height: 0)

    convenience init(callback: @escaping (JSON?) -> Void) {
        self.init(nibName: nil, bundle: nil)

        self.webview = RichTextEditorWebView()
        self.webview.initialize { [weak self] (json: JSON?) in
            callback(json)

            // make sure scroll is still fine on every change
            self?.ensureCursorVisible()
        }
        self.view = self.webview

        self.webview.onHeightChange = { [weak self] (height) in
            self?.ensureCursorVisible()
        }
    }

    override func viewDidLoad() {
        KeyboardUtils.sharedInstance.addWillShowObserver(self, selector: #selector(handleKeyboardWillShow))
        KeyboardUtils.sharedInstance.addWillHideObserver(self, selector: #selector(handleKeyboardWillHide))
    }

    func delegateScrollTo(_ outerScroll: UIScrollView) {
        self.outerScroll = outerScroll
        self.webview.scrollView.isScrollEnabled = false
        self.webview.setContentCompressionResistancePriority(UILayoutPriority.defaultHigh, for: .vertical)
    }

    func setContent(_ content: String) {
        self.webview.setContent(content)
    }

    @objc func handleKeyboardWillShow(_ notification: Notification) {
        if let _ = ((notification as NSNotification).userInfo?[UIResponder.keyboardFrameBeginUserInfoKey] as? NSValue)?.cgRectValue {
            self.replaceKeyboardInputAccessoryView()
        }

        self.webview.checkSize()

        // https://stackoverflow.com/questions/31774006/how-to-get-height-of-keyboard#33130819
        if let keyboardFrame: NSValue = notification.userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue {
            let keyboardRectangle = keyboardFrame.cgRectValue
            self.keyboardRect = keyboardRectangle
        }

        // change inset of scroll view to accommodate for keyboard
        if let outerScroll = self.outerScroll {
            UIView.animate(withDuration: 0.3) {
                let contentInsets = UIEdgeInsets(top: 0, left: 0, bottom: self.keyboardRect.height, right: 0)
                outerScroll.contentInset = contentInsets
                outerScroll.scrollIndicatorInsets = contentInsets
            }
        }

        self.ensureCursorVisible()
    }

    @objc func handleKeyboardWillHide(_ notification: Notification) {
        self.webview.checkSize()

        if let outerScroll = self.outerScroll {
            UIView.animate(withDuration: 0.3) {
                outerScroll.contentInset = .zero
                outerScroll.scrollIndicatorInsets = .zero
            }
        }
    }

    private func ensureCursorVisible() {
        if self.outerScroll == nil {
            // only supported with outer scroll
            return
        }

        self.webview.getCursorPosition { [weak self] point in
            self?.ensureCursorVisible(webviewCursor: point)
        }
    }

    private func ensureCursorVisible(webviewCursor: CGPoint) {
        guard let outerScroll = self.outerScroll else {
            // only supported with outer scroll
            return
        }

        if webviewCursor.y == 0 {
            // no cursor from webview yet, we wait for it before doing anything
            print("RichTextEditor > Can't get webview cursor. Bailing out.")
            return
        }

        let webviewTop = self.view.frame.minY

        // y position of the cursor in the scrolled frame (which is offset under the nav bar)
        let cursorFramePosition = (webviewTop + webviewCursor.y) - (outerScroll.contentOffset.y + outerScroll.adjustedContentInset.top)
        print("RichTextEditor > webviewTop=\(webviewTop) webviewCursor=\(webviewCursor.y) outerOffset=\(outerScroll.contentOffset.y) insetTop=\(outerScroll.adjustedContentInset.top) insetBottom=\(outerScroll.adjustedContentInset.bottom)")

        // y position of the cursor on the screen
        let cursorScreenPosition = cursorFramePosition + outerScroll.adjustedContentInset.top

        // smallest y position that is visible (after nav bar)
        let topVisibleY = outerScroll.adjustedContentInset.top

        // y position of the tab bar on the screen (we are adding keyboard height to inset, when it shows, we need to subtract it)
        let bottomBarY = UIScreen.main.bounds.height - (outerScroll.adjustedContentInset.bottom - self.keyboardRect.height)

        // highest y position that is visible (keyboard OR tab bar)
        var bottomVisibleY = self.keyboardRect.minY
        if bottomVisibleY > bottomBarY {
            // keyboard is smaller than bottom bar
            bottomVisibleY = bottomBarY
        }

        print("RichTextEditor > top=\(topVisibleY) cursor=\(cursorScreenPosition) bottom=\(bottomVisibleY)")

        if cursorScreenPosition - 20 < topVisibleY {
            // cursor is bellow top nav bar
            let diff = topVisibleY - cursorScreenPosition + 40
            print("RichTextEditor > Cursor is below nav bar. Scrolling down by \(diff).")
            outerScroll.setContentOffset(CGPoint(x: outerScroll.contentOffset.x, y: outerScroll.contentOffset.y - diff), animated: true)

        } else if cursorScreenPosition + 20 > bottomVisibleY {
            // cursor is bellow keyboard or bottom tab bar
            let diff = cursorScreenPosition - bottomVisibleY + 40
            print("RichTextEditor > Cursor is below visible bottom. Scrolling up by \(diff).")
            outerScroll.setContentOffset(CGPoint(x: outerScroll.contentOffset.x, y: outerScroll.contentOffset.y + diff), animated: true)
        }
    }

    deinit {
        print("RichTextEditor > Deinit")
        KeyboardUtils.sharedInstance.removeObserver(self)
    }
}

extension RichTextEditor {
    // from http://stackoverflow.com/questions/30312525/replace-inputaccessoryview-of-keyboard-in-uiwebview-in-swift

    func addNewAccessoryView(_ oldAccessoryView: UIView) {
        let newAccessoryView = RichTextEditorToolsView(frame: oldAccessoryView.frame)
        newAccessoryView.editor = self
        oldAccessoryView.addSubview(newAccessoryView)

        // so that we hide the < > controls
        // this is platform specific... but only way easy
        newAccessoryView.backgroundColor = UIColor.systemGray4

        newAccessoryView.snp.makeConstraints { (make) in
            make.left.equalTo(oldAccessoryView.snp.left)
            make.top.equalTo(oldAccessoryView.snp.top).offset(1)
            make.height.equalTo(oldAccessoryView.snp.height)
            make.right.equalTo(oldAccessoryView.snp.right)
        }
    }

    func traverseSubViews(_ vw: UIView) -> UIView {
        if (vw.description.hasPrefix("<UIWebFormAccessory")) {
            return vw
        }
        for subview in vw.subviews as [UIView?] {
            if ((subview?.subviews.count)! > 0) {
                let subvw = self.traverseSubViews(subview!)
                if (subvw.description.hasPrefix("<UIWebFormAccessory")) {
                    return subvw
                }
            }
        }
        return UIView()
    }

    func replaceKeyboardInputAccessoryView() {
        // locate accessory view
        let windowCount = UIApplication.shared.windows.count
        if (windowCount < 2) {
            return
        }

        let tempWindow: UIWindow = UIApplication.shared.windows[1] as UIWindow
        let accessoryView: UIView = traverseSubViews(tempWindow)
        if (accessoryView.description.hasPrefix("<UIWebFormAccessory")) {
            // Found the inputAccessoryView UIView
            if (accessoryView.subviews.count > 0) {
                self.addNewAccessoryView(accessoryView)
            }
        }
    }
}

fileprivate class RichTextEditorWebView: HybridWebView {
    func initialize(_ callback: @escaping (JSON?) -> Void) {
        self.initialize("html-editor", callback: callback)

        // take over scroll delegate when managed externally
        if self.scrollView.isScrollEnabled {
            self.scrollView.delegate = self
        }

        // snippet from https://stackoverflow.com/questions/11126047/find-y-coordinate-for-cursor-position-in-div-in-uiwebview
        self.evaluateJavaScript("""
                                function getCaretClientPosition() {
                                    var x = 0, y = 0;
                                    var sel = window.getSelection();
                                    if (sel.rangeCount) {
                                        var range = sel.getRangeAt(0);
                                        if (range.getClientRects) {
                                            var rects = range.getClientRects();
                                            if (rects.length > 0) {
                                                x = rects[0].left;
                                                y = rects[0].top;
                                            }
                                        }
                                    }
                                    return { x: x, y: y };
                                }
                                """)
    }

    func getCursorPosition(_ callback: @escaping (CGPoint) -> ()) {
        self.evaluateJavaScript("getCaretClientPosition()") { any, error in
            guard error == nil,
                  let dict = any as? Dictionary<String, Any>,
                  let x = dict["x"] as? Double,
                  let y = dict["y"] as? Double else {
                return
            }

            callback(CGPoint(x: x, y: y))
        }
    }

    func scrollViewDidScroll(_ scrollView: UIScrollView) {
        // prevent any scroll if it's disabled
        // this may happen if keyboard is shown in a text field
        if (!self.scrollView.isScrollEnabled) {
            scrollView.contentOffset = CGPoint(x: 0, y: 0)
        }
    }

    func setContent(_ content: String) {
        self.setData(["content": content as AnyObject])
    }
}

fileprivate class RichTextEditorToolsView: UIView {
    weak var editor: RichTextEditor!

    let buttons = [
        ("bold", FontAwesome.bold),
        ("strikethrough", FontAwesome.strikethrough),
        ("header-toggle", FontAwesome.heading),
        ("list-ul", FontAwesome.listUl),
        ("list-ol", FontAwesome.listOl),
        ("outdent", FontAwesome.outdent),
        ("indent", FontAwesome.indent),
    ]

    override init(frame: CGRect) {
        super.init(frame: frame)
        self.createView()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    func createView() {
        let buttonViews = buttons.enumerated().map { (i, tup) -> UIButton in
            let (_, icon) = tup
            let button = UIButton()
            let img = UIImage.fontAwesomeIcon(name: icon, style: .solid, textColor: UIColor.label, size: CGSize(width: 25, height: 25))
            button.tag = i
            button.addTarget(self, action: #selector(handleButtonTouch), for: .touchUpInside)
            button.setImage(img, for: UIControl.State())
            return button
        }

        let stack = UIStackView(arrangedSubviews: buttonViews)
        stack.alignment = .leading
        stack.axis = .horizontal
        stack.spacing = 15
        self.addSubview(stack)
        stack.snp.makeConstraints { (make) in
            make.left.equalTo(self.snp.left).offset(20)
            make.centerY.equalTo(self.snp.centerY)
        }

        let closeButton = UIButton()
        closeButton.setTitle("Done", for: .normal)
        closeButton.setTitleColor(UIColor.label, for: .normal)
        closeButton.addTarget(self, action: #selector(handleCloseKeyboard), for: .touchUpInside)
        self.addSubview(closeButton)
        closeButton.snp.makeConstraints { (make) in
            make.right.equalToSuperview().offset(-20)
            make.centerY.equalToSuperview()
        }
    }

    @objc func handleButtonTouch(_ sender: UIButton) {
        let (name, _) = self.buttons[sender.tag]
        self.editor.webview.setData(["action": name as AnyObject])
    }

    @objc func handleCloseKeyboard(_ sender: UIButton) {
        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
    }
}
