Vagrant.configure(2) do |config|

  # We use Ubuntu instead of Debian because the image comes with two-way
  # shared folder support by default.
  UBUNTU = 'hashicorp/bionic64'

  # The main VM is the one used for development and testing.
  config.vm.define(:specsheet, primary: true) do |config|

    config.vm.provider :vmware_desktop do |v|
      v.vmx['memsize'] = '2048'
      v.vmx['numvcpus'] = `nproc`.chomp
    end

    # https://github.com/hashicorp/vagrant/issues/10575
    config.vm.network :forwarded_port, guest: 22, host: 2498, id: 'ssh'

    config.vm.box = UBUNTU
    config.vm.hostname = 'specsheet'
    developer = 'vagrant'


    # Install the dependencies needed to build, as quietly as
    # apt can do. (https://askubuntu.com/a/615016)
    config.vm.provision :shell, privileged: true, inline: <<-EOF
      trap 'exit' ERR
      apt-get -qq update
      apt-get -qq install -o=Dpkg::Use-Pty=0 \
        curl ruby build-essential \
        fish zsh bash bash-completion
    EOF


    # Install Rust.
    # This is done as the user, not root, because it’s the user
    # who actually uses it. Sent to /dev/null because the progress
    # bar produces a ton of output.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      if hash rustc &>/dev/null; then
        echo "Rust is already installed"
      else
        set -xe
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
      fi
    EOF


    # Use a different ‘target’ directory on the VM than on the host.
    # By default it just uses the one in /vagrant/target, which can
    # cause problems if it has different permissions than the other
    # directories, or contains compilation results from the host.
    config.vm.provision :shell, privileged: true, inline: <<-EOF
      echo "Reassigning target directory..."
      echo 'PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/home/#{developer}/.cargo/bin"' > /etc/environment
      echo 'CARGO_TARGET_DIR="/home/#{developer}/target"'                                                     >> /etc/environment
    EOF


    # Link the completion files so they can be used and tested.
    config.vm.provision :shell, privileged: true, inline: <<-EOF
      set -xe

      test -h /usr/local/bin/specsheet \
        || ln -s /vagrant/target/debug/specsheet /usr/local/bin/specsheet

      test -h /etc/bash_completion.d/specsheet \
        || ln -s /vagrant/completions/specsheet.bash /etc/bash_completion.d/specsheet

      test -h /usr/share/zsh/vendor-completions/_specsheet \
        || ln -s /vagrant/completions/specsheet.zsh /usr/share/zsh/vendor-completions/_specsheet

      test -h /usr/share/fish/completions/specsheet.fish \
        || ln -s /vagrant/completions/specsheet.fish /usr/share/fish/completions/specsheet.fish
    EOF


    # Configure the environment for the tests.
    config.vm.provision :shell, privileged: true,
      path: 'xtests/set-up-environment.sh'
  end
end
