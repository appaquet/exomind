import classNames from 'classnames';
import React, { ChangeEvent } from 'react';
import './input-modal.less';

interface IProps {
    text: string;
    initialValue?: string;
    onDone: (value: string | null, cancelled: boolean) => void;
}

interface IState {
    value: string;
}

export default class InputModal extends React.Component<IProps, IState> {
    inputRef: React.RefObject<HTMLInputElement> = React.createRef();

    constructor(props: IProps) {
        super(props);

        this.state = {
            value: props.initialValue ?? ''
        };
    }

    componentDidMount(): void {
        this.inputRef.current.focus();
    }

    render(): React.ReactNode {
        const classes = classNames({
            'input-modal': true,
        });

        return (
            <div className={classes}>
                <div className="text">{this.props.text}</div>
                <div className="value">
                    <input type="text"
                        ref={this.inputRef}
                        value={this.state.value}
                        onChange={this.onValueChange.bind(this)}
                        onKeyDown={this.onKeyDown.bind(this)}
                    />
                </div>

                <div className="buttons">
                    <button onClick={this.onCancel.bind(this)}>Cancel</button>
                    <button onClick={this.onDone.bind(this)}>Done</button>
                </div>
            </div>
        );
    }

    private onValueChange(e: ChangeEvent<HTMLInputElement>): void {
        this.setState({
            value: e.target.value
        });
    }

    private onKeyDown(e: KeyboardEvent): void {
        if (e.key == 'Enter') {
            this.props.onDone(this.state.value, false);
        }
    }

    private onCancel(): void {
        this.props.onDone(null, true);
    }

    private onDone(): void {
        this.props.onDone(this.state.value, false);
    }
}