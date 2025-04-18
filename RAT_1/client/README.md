# Client

## 使用方法

```
$ cargo run -p client -- help
A client

Usage: client.exe <COMMAND>

Commands:
  agents   List all agents
  jobs     List all job
  exec     Execute a command to the agent by agent id
  signing  Generate a new Signing keypair
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

See 'client help <command>' for more information on a specific command.
```

```
$ cargo run -p client -- help exec
Execute a command to the agent by agent id

Usage: client.exe exec --agent <AGENT> --command <COMMAND>

Options:
  -a, --agent <AGENT>      The agent id to execute the command on
      --command <COMMAND>  The command to execute, with its arguments
  -h, --help               Print help
```

## 注意事项
如果需要变更client的签名密钥，可以使用signing命令生成新的密钥对

```
$  cargo run -p client -- signing
private key: hLe9lT83ck+9zBtnRrcJ9CGweZwP1egKn0T3i2ceJN0=
public key: vTa+uzLgo/AFVoudm2GCkPt+P8ZFsNUIA72KY6x8LcQ=
```

然后修改client和agent的密钥配置以及server的环境变量，修改client配置文件中的client签名私钥为生成的private key，修改agent配置文件中的client签名公钥为生成的public key，server的环境变量CLIENT_SIGNING_PUBLIC_KEY修改为生成的public key

