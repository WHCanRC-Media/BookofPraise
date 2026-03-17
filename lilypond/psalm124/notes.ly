melody = \relative c'' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f2 g4 a4 bes2 a2 g4 f4 f4 e4 f2 r2 \break

  % Line 2
  a2 bes4 c4 d2 c2 bes4 a4 g2 f2 e2 r2 \break

  % Line 3
  c2 f4 f4 e2 f2 g4 bes2 a2 g4 a2 r2 \break

  % Line 4
  c2 c4 bes4 a2 g2 a4 c4 c4 bes4 c2 r2 \break

  % Line 5
  a2 g4 f4 e2 f2 g4 bes4 a2 g2 f1 \bar "|."
}
