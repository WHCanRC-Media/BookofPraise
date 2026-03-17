melody = \relative c'' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a2 f2 g2 a4 a4 g4 f4 a2 bes2 c4 \break

  % Line 2
  a4 a4 g2 f2 e4 f2 r2 \break

  % Line 3
  f2 f4 g4 a4 a4 c4 bes4 a2 g2 f2 e4 \break

  % Line 4
  f4 g4 a4 bes2 a2 g2 f2 r2 \break

  % Line 5
  a2 a4 bes4 c2 bes2 a4 g4 f4 g4 a2 g4 \break

  % Line 6
  bes4 a4 g4 f4 e4 d2 c2 r2 \break

  % Line 7
  f2 a4 bes4 c2 c2 a4 a4 f2 g2 a4 \break

  % Line 8
  a4 g4 bes4 a2 g2 f1 \bar "|."
}
