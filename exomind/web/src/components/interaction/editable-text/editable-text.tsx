
import React, { MouseEvent } from 'react';
import './editable-text.less';

interface IProps {
  text: string;
  editText?: string;
  placeholder?: string;
  multiline?: boolean;
  doubleClick?: boolean;
  onChange?: (value: string) => void;
  initializeEditing?: boolean;
  onBound?: (ed: EditableText) => void;
  onFocus?: () => void;
  onBlur?: () => void;
}

interface IState {
  editing: boolean;
  value: string;
}

interface InputElement {
  value: string;
  focus(): void;
  blur(): void;
  select(): void;
}

export default class EditableText extends React.Component<IProps, IState> {
  private inputRef: React.RefObject<Element & InputElement> = React.createRef();

  constructor(props: IProps) {
    super(props);

    this.state = {
      editing: !!props.initializeEditing,
      value: props.text || ''
    };
  }

  componentDidUpdate(prevProps: IProps): void {
    if (this.state.editing) {
      this.ensureFocus();
      return;
    }

    if (this.props.text != this.state.value) {
      this.setState({
        value: this.props.text
      });
    }

    if (this.props.initializeEditing && this.props.initializeEditing != prevProps.initializeEditing) {
      this.focus();
    }
  }

  componentDidMount(): void {
    if (this.state.editing) {
      this.ensureFocus();
    }
    this.props.onBound?.(this);
  }

  render(): React.ReactNode {
    if (this.state.editing) {
      if (this.props.multiline) {
        return this.renderMultiEdit();
      } else {
        return this.renderSingleEdit();
      }
    } else {
      return this.renderReadOnly();
    }
  }

  private renderReadOnly(): React.ReactNode {
    const placeholder = this.props.placeholder || 'Click to change';
    const singleClick = !this.props.doubleClick;
    const value = !this.state.value ? <span className="empty">{placeholder}</span> : this.state.value;
    return (
      <span
        className="editable-text"
        onClick={singleClick ? this.handleReadClick : null}
        onDoubleClick={this.handleReadClick}>
        {value}
      </span>
    );
  }

  private renderSingleEdit(): React.ReactNode {
    return (
      <span className="editable-text">
        <input
          type="text"
          ref={this.inputRef as React.RefObject<HTMLInputElement>}
          onBlur={this.handleEditBlur}
          onChange={this.handleEditChange}
          onKeyDown={this.handleEditKeyPress}
          value={this.state.value}
          onClick={this.handleEditClick}
        />
      </span>
    );
  }

  private renderMultiEdit(): React.ReactNode {
    return (
      <span className="editable-text">
        <textarea
          ref={this.inputRef as React.RefObject<HTMLTextAreaElement>}
          onBlur={this.handleEditBlur}
          onChange={this.handleEditChange}
          onKeyDown={this.handleEditKeyPress}
          value={this.state.value} />
      </span>
    );
  }

  focus(): void {
    this.setState({
      editing: true
    });

    setTimeout(() => {
      this.ensureFocus();
    }, 500);
  }

  private handleReadClick = (e: MouseEvent) => {
    if (e.metaKey || e.ctrlKey || e.shiftKey) {
      // prevent click when multi-select
      return;
    }

    this.setState({
      editing: true,
      value: this.props.editText || this.props.text,
    });
    e.stopPropagation();
  };

  private handleEditChange = (event: React.ChangeEvent<InputElement>) => {
    this.setState({
      value: event.target.value
    });
  };

  private handleEditKeyPress = (event: React.KeyboardEvent) => {
    if (event.key == 'Escape' || (!this.props.multiline && event.key == 'Enter')) {
      event.stopPropagation();
      this.editFinish();
    }
  };

  private handleEditBlur = () => {
    this.editFinish();
  };

  private handleEditClick = (event: MouseEvent) => {
    // necessary since we may be in the list of entity and don't want the event to go further
    event.stopPropagation();
  };

  private editFinish() {
    this.setState({
      editing: false
    });

    if (this.props.onChange) {
      this.props.onChange(this.state.value);
    }

    this.props.onBlur?.();
  }

  private ensureFocus() {
    const element = this.inputRef.current;
    if (!element) {
      return;
    }

    if (element != document.activeElement) {
      element.focus();
      element.select();
    }

    this.props.onFocus?.();
  }
}
