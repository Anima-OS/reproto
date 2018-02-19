import * as React from 'react';
import AceEditor from 'react-ace';

export interface InputProps {
  format: string;
  value: string;
  onChange: (value: string) => void;
}

export class Input extends React.Component<InputProps, {}> {
  render() {
    return (
      <AceEditor
        showGutter={true}
        mode={this.props.format}
        theme="monokai"
        width="100%"
        height="100%"
        value={this.props.value}
        onChange={this.props.onChange.bind(this)}
        />
    );
  }
}