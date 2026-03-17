melody = \relative c'' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g2 g4 f4 e2 d2 f4 a2 g2 fis4 g2 r2 \break

  % Line 2
  a2 c4 bes4 a2 g2 a4 bes4 g4 a4 g2 f2 r2 \break

  % Line 3
  g2 g4 f4 g2 a2 c4 g4 a2 bes2 a2 r2 \break

  % Line 4
  a2 a4 g4 f2 d2 f4 g4 a4 g2 fis4 g1 \bar "|."
}
