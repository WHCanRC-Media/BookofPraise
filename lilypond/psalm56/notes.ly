melody = \relative c'' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f2 d4 c4 f2 f4 g2 a4 bes2 a2 g2 r2 \break

  % Line 2
  f2 g4 g4 a2 c2 c4 bes4 g2 bes2 a2 r2 \break

  % Line 3
  g2 a4 g4 e2 e2 f4 e4 d4 d4 c2 \break

  % Line 4
  c2 bes4 a4 g4 f4 g2 f2 r2 \break

  % Line 5
  f2 bes4 bes4 a2 g2 a4 bes4 c4 c4 d2 c2 r2 c2 f,4 g4 a2 f2 g4 a4 bes4 a4 g2 a2 r2 \break

  % Line 6
  a2 a4 a4 g2 a2 bes4 a4 g4 f4 e2 c2 \break

  % Line 7
  c2 bes4 a4 g4 g4 f1 \bar "|."
}
