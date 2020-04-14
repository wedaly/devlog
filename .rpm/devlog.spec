%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: devlog
Summary: devlog is a command-line tool for tracking your day-to-day software development work.
Version: @@VERSION@@
Release: @@RELEASE@@
License: MIT
Group: Applications/System
Source0: %{name}-%{version}.tar.gz
URL: https://devlog-cli.org

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

Requires: nano

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
